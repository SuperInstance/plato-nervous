//! Ollama Integration for PLATO Nervous System
//!
//! Replaces the SIMULATED nano model with real LLM inference via ollama.
//! Connects to a local ollama server at localhost:11434.
//!
//! Two model tiers:
//! - Nano (350M): per-room anomaly detection (e.g. liquid-350m)
//! - Fleet (1.2B): cross-room coordination (e.g. liquid-1.2b)

use crate::{ResolutionLayer, SensorReading, Tile, TileType};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use uuid::Uuid;

// ── Error Type ───────────────────────────────────────────────────────

#[derive(Debug)]
pub enum OllamaError {
    Http(reqwest::Error),
    Parse(String),
    Timeout,
    ModelNotAvailable(String),
    ServerOffline,
}

impl std::fmt::Display for OllamaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OllamaError::Http(e) => write!(f, "HTTP error: {}", e),
            OllamaError::Parse(msg) => write!(f, "Parse error: {}", msg),
            OllamaError::Timeout => write!(f, "Request timed out"),
            OllamaError::ModelNotAvailable(m) => write!(f, "Model '{}' not available", m),
            OllamaError::ServerOffline => write!(f, "Ollama server offline"),
        }
    }
}

impl std::error::Error for OllamaError {}

impl From<reqwest::Error> for OllamaError {
    fn from(e: reqwest::Error) -> Self {
        OllamaError::Http(e)
    }
}

// ── Ollama API Types ─────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    #[serde(default)]
    #[allow(dead_code)]
    done: bool,
    #[serde(default)]
    #[allow(dead_code)]
    eval_duration: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    total_duration: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct OllamaListResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, Deserialize)]
struct OllamaModelInfo {
    name: String,
    #[allow(dead_code)]
    modified_at: String,
    #[allow(dead_code)]
    size: u64,
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u64>,
}

// ── OllamaClient ─────────────────────────────────────────────────────

/// HTTP client for the ollama API at localhost:11434
#[derive(Debug, Clone)]
pub struct OllamaClient {
    #[allow(dead_code)]
    base_url: String,
    client: reqwest::Client,
    #[allow(dead_code)]
    timeout_secs: u64,
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new("http://localhost:11434".to_string(), 30)
    }
}

impl OllamaClient {
    pub fn new(base_url: String, timeout_secs: u64) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .expect("Failed to build reqwest client");
        Self {
            base_url,
            client,
            timeout_secs,
        }
    }

    /// Generate a response from the given model and prompt.
    /// Returns (response_text, latency_ms).
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        options: Option<GenerateOptions>,
    ) -> Result<(String, u64), OllamaError> {
        let req = OllamaGenerateRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            stream: Some(false),
            options: options.map(|o| OllamaOptions {
                temperature: o.temperature,
                num_predict: o.max_tokens,
                top_p: o.top_p,
                seed: o.seed,
            }),
        };

        let start = Instant::now();
        let resp = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE
            || resp.status() == reqwest::StatusCode::BAD_GATEWAY
        {
            return Err(OllamaError::ServerOffline);
        }

        let body: OllamaGenerateResponse = resp.json().await?;
        let latency = start.elapsed().as_millis() as u64;

        Ok((body.response, latency))
    }

    /// Generate a structured (JSON) response. Expects the model to output valid JSON.
    pub async fn generate_structured<T: serde::de::DeserializeOwned>(
        &self,
        model: &str,
        prompt: &str,
        options: Option<GenerateOptions>,
    ) -> Result<(T, u64), OllamaError> {
        let (text, latency) = self.generate(model, prompt, options).await?;

        // Try to find JSON in the response (models sometimes wrap in markdown)
        let json_str = extract_json(&text)?;

        let parsed: T = serde_json::from_str(&json_str)
            .map_err(|e| OllamaError::Parse(format!("JSON parse error: {} — raw: {}", e, &text[..text.len().min(200)])))?;

        Ok((parsed, latency))
    }

    /// List all models available on the ollama server.
    pub async fn list_models(&self) -> Result<Vec<String>, OllamaError> {
        let resp = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(OllamaError::ServerOffline);
        }

        let body: OllamaListResponse = resp.json().await?;
        Ok(body.models.into_iter().map(|m| m.name).collect())
    }

    /// Ping the server to check availability.
    pub async fn is_available(&self) -> bool {
        self.client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}

// ── GenerateOptions ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f64>,
    pub seed: Option<u64>,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            temperature: Some(0.1),
            max_tokens: Some(128),
            top_p: Some(0.9),
            seed: None,
        }
    }
}

// ── OllamaModelConfig ────────────────────────────────────────────────

/// Configuration for an ollama model used in the PLATO pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModelConfig {
    /// The model name on the ollama server (e.g. "liquid-350m", "liquid-1.2b")
    pub model_name: String,
    /// Few-shot prompt template. Placeholders: {sensor_id}, {value}, {unit},
    /// {normal_min}, {normal_max}, {examples}
    pub prompt_template: String,
    /// Confidence threshold — below this, the signal escalates to the next layer
    pub confidence_threshold: f64,
    /// Temperature for generation (lower = more deterministic)
    pub temperature: f64,
    /// Maximum tokens to generate
    pub max_tokens: u32,
}

impl OllamaModelConfig {
    /// Default config for a nano model (350M class).
    pub fn nano_default(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            prompt_template: DEFAULT_NANO_TEMPLATE.to_string(),
            confidence_threshold: 0.7,
            temperature: 0.1,
            max_tokens: 64,
        }
    }

    /// Default config for a fleet coordinator (1.2B class).
    pub fn fleet_default(model_name: &str) -> Self {
        Self {
            model_name: model_name.to_string(),
            prompt_template: DEFAULT_FLEET_TEMPLATE.to_string(),
            confidence_threshold: 0.6,
            temperature: 0.2,
            max_tokens: 128,
        }
    }
}

// ── Default Prompt Templates ─────────────────────────────────────────

const DEFAULT_NANO_TEMPLATE: &str = r#"You are a room-level anomaly detection system.
Given a sensor reading, classify it as STATUS or ALERT and output JSON.

Rules:
- Values within normal range → STATUS, high confidence
- Values near the boundary (within 15% of edge) → STATUS, lower confidence (0.5-0.7)
- Values outside normal range → ALERT, moderate confidence (0.4-0.6)
- Extreme values (far outside range) → ALERT, low confidence (0.1-0.3) — escalate

Examples:
{sensor_id}={value}{unit} normal:{normal_min}-{normal_max}

Respond ONLY with valid JSON:
{"classification": "STATUS"|"ALERT", "confidence": 0.0-1.0, "reason": "short explanation"}

Reading: {sensor_id}={value}{unit} (normal range: {normal_min}-{normal_max})"#;

const DEFAULT_FLEET_TEMPLATE: &str = r#"You are a cross-room fleet coordinator.
Several rooms reported readings at the same time. Determine if they're related.

Readings:
{readings}

Respond ONLY with valid JSON:
{"related": true|false, "root_cause": "description or null", "coordination_tile": "action needed", "confidence": 0.0-1.0}"#;

// ── Helper: Extract JSON from model output ───────────────────────────

fn extract_json(text: &str) -> Result<String, OllamaError> {
    let trimmed = text.trim();

    // Try direct parse
    if trimmed.starts_with('{') {
        return Ok(trimmed.to_string());
    }

    // Try to find a JSON block if wrapped in markdown ```json ... ```
    if let Some(start) = trimmed.find("{") {
        if let Some(end) = trimmed.rfind("}") {
            return Ok(trimmed[start..=end].to_string());
        }
    }

    Err(OllamaError::Parse(format!(
        "No JSON object found in response: {}",
        &text[..text.len().min(150)]
    )))
}

// ── RealNanoModel ────────────────────────────────────────────────────

/// Wraps `OllamaClient` with the nano model config to replace the simulated
/// `NanoModel`. Reads a sensor reading, builds a prompt, calls ollama,
/// and parses the result into a (Tile, confidence) tuple.
#[derive(Debug, Clone)]
pub struct RealNanoModel {
    pub client: OllamaClient,
    pub config: OllamaModelConfig,
    pub tiles_produced: usize,
    pub avg_confidence: f64,
    pub avg_latency_ms: f64,
}

impl RealNanoModel {
    pub fn new(client: OllamaClient, config: OllamaModelConfig) -> Self {
        Self {
            client,
            config,
            tiles_produced: 0,
            avg_confidence: 0.5,
            avg_latency_ms: 0.0,
        }
    }

    /// Format the few-shots template with the reading's values.
    fn format_prompt(&self, reading: &SensorReading, examples: &[NanoExample]) -> String {
        let examples_str: String = examples
            .iter()
            .map(|ex| {
                format!(
                    "Input: {}={:.1}{} normal:{:.1}-{:.1} → {} (confidence {:.2})",
                    ex.sensor_id, ex.value, ex.unit,
                    ex.normal_min, ex.normal_max,
                    ex.classification, ex.confidence,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        self.config
            .prompt_template
            .replace("{sensor_id}", &reading.sensor_id)
            .replace("{value}", &format!("{:.1}", reading.value))
            .replace("{unit}", &reading.unit)
            .replace("{normal_min}", &format!("{:.1}", reading.normal_min))
            .replace("{normal_max}", &format!("{:.1}", reading.normal_max))
            .replace("{examples}", &examples_str)
    }

    /// Infer from a sensor reading. Returns None if confidence is below threshold
    /// (meaning the signal should escalate to the next layer).
    pub async fn infer(&mut self, reading: &SensorReading) -> Option<(Tile, f64)> {
        // Build examples for few-shot
        let examples = default_examples();
        let prompt = self.format_prompt(reading, &examples);

        let options = GenerateOptions {
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            ..Default::default()
        };

        let (response, latency) = self
            .client
            .generate(&self.config.model_name, &prompt, Some(options))
            .await
            .ok()?;

        // Parse the JSON response
        let parsed: NanoResponse = parse_nano_response(&response).ok()?;

        if parsed.confidence >= self.config.confidence_threshold {
            self.tiles_produced += 1;
            let alpha = 1.0 / self.tiles_produced as f64;
            self.avg_confidence =
                self.avg_confidence * (1.0 - alpha) + parsed.confidence * alpha;
            self.avg_latency_ms =
                self.avg_latency_ms * (1.0 - alpha) + latency as f64 * alpha;

            let tile_type = match parsed.classification.to_uppercase().as_str() {
                "ALERT" => TileType::Alert,
                "PREDICTION" => TileType::Prediction,
                _ => TileType::Status,
            };

            Some((
                Tile {
                    id: Uuid::new_v4(),
                    room_id: reading.room_id.clone(),
                    tile_type,
                    content: parsed.reason,
                    confidence: parsed.confidence,
                    resolved_by: ResolutionLayer::NanoModel,
                    timestamp_ms: reading.timestamp_ms,
                    sensor_reading: Some(reading.clone()),
                },
                parsed.confidence,
            ))
        } else {
            None
        }
    }
}

/// The expected JSON structure from the nano model.
#[derive(Debug, Deserialize)]
struct NanoResponse {
    classification: String,
    confidence: f64,
    reason: String,
}

fn parse_nano_response(text: &str) -> Result<NanoResponse, OllamaError> {
    let json_str = extract_json(text)?;
    serde_json::from_str(&json_str)
        .map_err(|e| OllamaError::Parse(format!("NanoResponse parse: {} — raw: {}", e, &json_str)))
}

/// Default few-shot examples for the nano model.
struct NanoExample {
    sensor_id: String,
    value: f64,
    unit: String,
    normal_min: f64,
    normal_max: f64,
    classification: String,
    confidence: f64,
}

fn default_examples() -> Vec<NanoExample> {
    vec![
        NanoExample {
            sensor_id: "temp".into(),
            value: 22.0,
            unit: "C".into(),
            normal_min: 15.0,
            normal_max: 30.0,
            classification: "STATUS".into(),
            confidence: 0.95,
        },
        NanoExample {
            sensor_id: "temp".into(),
            value: 29.5,
            unit: "C".into(),
            normal_min: 15.0,
            normal_max: 30.0,
            classification: "STATUS".into(),
            confidence: 0.65,
        },
        NanoExample {
            sensor_id: "rpm".into(),
            value: 3200.0,
            unit: "rpm".into(),
            normal_min: 1000.0,
            normal_max: 3000.0,
            classification: "ALERT".into(),
            confidence: 0.55,
        },
        NanoExample {
            sensor_id: "oil_pressure".into(),
            value: 15.0,
            unit: "psi".into(),
            normal_min: 30.0,
            normal_max: 80.0,
            classification: "ALERT".into(),
            confidence: 0.25,
        },
    ]
}

// ── FleetCoordinatorResponse ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct FleetCoordinatorResponse {
    #[allow(dead_code)]
    related: bool,
    root_cause: Option<String>,
    coordination_tile: Option<String>,
    confidence: f64,
}

// ── RealFleetModel ────────────────────────────────────────────────────

/// Wraps `OllamaClient` with the fleet model config for cross-room coordination.
/// Takes multiple sensor readings and determines if they're causally related.
#[derive(Debug, Clone)]
pub struct RealFleetModel {
    pub client: OllamaClient,
    pub config: OllamaModelConfig,
    pub coordination_count: usize,
}

impl RealFleetModel {
    pub fn new(client: OllamaClient, config: OllamaModelConfig) -> Self {
        Self {
            client,
            config,
            coordination_count: 0,
        }
    }

    /// Format the readings block for the fleet template.
    fn format_readings(&self, readings: &[SensorReading]) -> String {
        readings
            .iter()
            .map(|r| {
                format!(
                    "[{}] {}={:.1}{} (range: {:.1}-{:.1}, room: {})",
                    r.timestamp_ms, r.sensor_id, r.value, r.unit,
                    r.normal_min, r.normal_max, r.room_id,
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Analyze several sensor readings from different rooms and determine
    /// if they're related. If so, returns a coordination tile.
    pub async fn analyze(
        &mut self,
        readings: &[SensorReading],
    ) -> Option<(Tile, f64)> {
        if readings.is_empty() {
            return None;
        }

        let readings_str = self.format_readings(readings);
        let prompt = self
            .config
            .prompt_template
            .replace("{readings}", &readings_str);

        let options = GenerateOptions {
            temperature: Some(self.config.temperature),
            max_tokens: Some(self.config.max_tokens),
            ..Default::default()
        };

        let (response, _latency) = self
            .client
            .generate(&self.config.model_name, &prompt, Some(options))
            .await
            .ok()?;

        let parsed: FleetCoordinatorResponse = {
            let json_str = extract_json(&response).ok()?;
            serde_json::from_str(&json_str).ok()?
        };

        if parsed.confidence < self.config.confidence_threshold {
            return None;
        }

        // Combine the room IDs into a single room string
        let room_ids: Vec<&str> = readings.iter().map(|r| r.room_id.as_str()).collect();
        let room_id = room_ids.join("+");
        let content = parsed
            .coordination_tile
            .unwrap_or_else(|| parsed.root_cause.clone().unwrap_or_default());

        self.coordination_count += 1;

        Some((
            Tile {
                id: Uuid::new_v4(),
                room_id,
                tile_type: TileType::Coordination,
                content,
                confidence: parsed.confidence,
                resolved_by: ResolutionLayer::FleetCoord,
                timestamp_ms: readings
                    .iter()
                    .map(|r| r.timestamp_ms)
                    .max()
                    .unwrap_or(0),
                sensor_reading: None,
            },
            parsed.confidence,
        ))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SensorReading;

    fn test_reading(sensor_id: &str, value: f64, min: f64, max: f64) -> SensorReading {
        SensorReading {
            sensor_id: sensor_id.to_string(),
            room_id: "engine-room".to_string(),
            value,
            unit: "units".to_string(),
            timestamp_ms: 1000,
            normal_min: min,
            normal_max: max,
        }
    }

    // ── Prompt Formatting Tests ────────────────────────────────────

    #[test]
    fn test_nano_prompt_format_contains_placeholders() {
        let config = OllamaModelConfig::nano_default("liquid-350m");
        let client = OllamaClient::default();
        let model = RealNanoModel::new(client, config);

        let reading = test_reading("temp", 25.0, 15.0, 30.0);
        let examples = default_examples();
        let prompt = model.format_prompt(&reading, &examples);

        // Should contain the sensor values
        assert!(prompt.contains("temp"), "prompt should contain sensor_id");
        assert!(prompt.contains("25.0"), "prompt should contain value");
        assert!(prompt.contains("15.0"), "prompt should contain normal_min");
        assert!(prompt.contains("30.0"), "prompt should contain normal_max");

        // Should contain few-shot examples
        assert!(prompt.contains("STATUS"), "prompt should contain examples");
        assert!(prompt.contains("ALERT"), "prompt should contain examples");
    }

    #[test]
    fn test_fleet_prompt_format() {
        let config = OllamaModelConfig::fleet_default("liquid-1.2b");
        let client = OllamaClient::default();
        let model = RealFleetModel::new(client, config);

        let readings = vec![
            test_reading("temp", 45.0, 15.0, 30.0),
            test_reading("rpm", 5000.0, 1000.0, 3000.0),
        ];
        let formatted = model.format_readings(&readings);

        assert!(formatted.contains("engine-room"), "should contain room ids");
        assert!(formatted.contains("45.0"), "should contain values");
        assert!(formatted.contains("5000.0"), "should contain values");
    }

    // ── Response Parsing Tests ─────────────────────────────────────

    #[test]
    fn test_parse_direct_json() {
        let response = r#"{"classification": "STATUS", "confidence": 0.95, "reason": "within normal range"}"#;
        let parsed = parse_nano_response(response).unwrap();
        assert_eq!(parsed.classification, "STATUS");
        assert!((parsed.confidence - 0.95).abs() < 0.01);
        assert_eq!(parsed.reason, "within normal range");
    }

    #[test]
    fn test_parse_json_in_markdown() {
        let response = r#"Here is the JSON:
```json
{"classification": "ALERT", "confidence": 0.45, "reason": "temperature too high"}
```
"#;
        let parsed = parse_nano_response(response).unwrap();
        assert_eq!(parsed.classification, "ALERT");
        assert!((parsed.confidence - 0.45).abs() < 0.01);
    }

    #[test]
    fn test_parse_json_with_extra_text() {
        let response = r#"I think the classification is ALERT. {"classification": "ALERT", "confidence": 0.55, "reason": "boundary exceeded"}"#;
        let parsed = parse_nano_response(response).unwrap();
        assert_eq!(parsed.classification, "ALERT");
    }

    #[test]
    fn test_parse_invalid_json_returns_error() {
        let response = "no JSON here at all";
        let parsed = parse_nano_response(response);
        assert!(parsed.is_err());
    }

    #[test]
    fn test_extract_json_direct() {
        let json = r#"{"a": 1}"#;
        let result = extract_json(json).unwrap();
        assert_eq!(result, r#"{"a": 1}"#);
    }

    #[test]
    fn test_extract_json_from_markdown_block() {
        let text = "```json\n{\"a\": 1}\n```";
        let result = extract_json(text).unwrap();
        assert_eq!(result, r#"{"a": 1}"#);
    }

    #[test]
    fn test_extract_json_surrounded_by_text() {
        let text = "Response: {\"a\": 1} -- done";
        let result = extract_json(text).unwrap();
        assert_eq!(result, r#"{"a": 1}"#);
    }

    // ── Full RealNanoModel Pipeline (mocked HTTP) ──────────────────

    /// A mock HTTP server that returns canned responses for testing.
    /// We use a lightweight approach: start a tiny HTTP server on a random port.
    fn start_mock_ollama() -> (String, std::sync::Arc<std::sync::atomic::AtomicBool>) {
        use std::sync::atomic::AtomicBool;
        use std::sync::Arc;

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base = format!("http://127.0.0.1:{}", port);
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();

        std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            for stream in listener.incoming() {
                if shutdown_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                if let Ok(mut stream) = stream {
                    use std::io::{Read, Write};
                    let mut buf = [0u8; 8192];
                    if stream.read(&mut buf).is_ok() {
                        // Return a mock JSON response
                        let body = r#"{"response": "{\"classification\": \"STATUS\", \"confidence\": 0.92, \"reason\": \"within normal range\"}", "done": true, "eval_duration": 50000000, "total_duration": 100000000}"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                }
            }
        });

        (base, shutdown)
    }

    #[tokio::test]
    async fn test_real_nano_model_with_mocked_ollama() {
        let (base_url, shutdown) = start_mock_ollama();

        let client = OllamaClient::new(base_url, 5);
        let config = OllamaModelConfig::nano_default("test-model");
        let mut model = RealNanoModel::new(client, config);

        let reading = test_reading("temp", 22.0, 15.0, 30.0);
        let result = model.infer(&reading).await;

        // Shut down the mock server
        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

        assert!(result.is_some(), "Expected a tile from mocked ollama");
        let (tile, confidence) = result.unwrap();
        assert_eq!(tile.tile_type, TileType::Status);
        assert!(confidence >= 0.7);
        assert_eq!(model.tiles_produced, 1);
    }

    #[tokio::test]
    async fn test_real_nano_model_low_confidence_returns_none() {
        // Mock returning a low-confidence response
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base = format!("http://127.0.0.1:{}", port);
        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let s = shutdown.clone();

        std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            for stream in listener.incoming() {
                if s.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                if let Ok(mut stream) = stream {
                    use std::io::{Read, Write};
                    let mut buf = [0u8; 8192];
                    if stream.read(&mut buf).is_ok() {
                        // Low confidence response (< 0.7 threshold)
                        let body = r#"{"response": "{\"classification\": \"ALERT\", \"confidence\": 0.35, \"reason\": \"outside normal range\"}", "done": true, "eval_duration": 40000000, "total_duration": 90000000}"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                }
            }
        });

        let client = OllamaClient::new(base, 5);
        let mut config = OllamaModelConfig::nano_default("test-model");
        config.confidence_threshold = 0.7; // Ensure threshold > 0.35
        let mut model = RealNanoModel::new(client, config);

        let reading = test_reading("temp", 35.0, 15.0, 30.0);
        let result = model.infer(&reading).await;

        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

        assert!(result.is_none(), "Low confidence should return None (escalate)");
        assert_eq!(model.tiles_produced, 0);
    }

    #[tokio::test]
    async fn test_real_fleet_model_with_mocked_ollama() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let base = format!("http://127.0.0.1:{}", port);
        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let s = shutdown.clone();

        std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            for stream in listener.incoming() {
                if s.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                if let Ok(mut stream) = stream {
                    use std::io::{Read, Write};
                    let mut buf = [0u8; 8192];
                    if stream.read(&mut buf).is_ok() {
                        let body = r#"{"response": "{\"related\": true, \"root_cause\": \"coolant pump failure in engine room\", \"coordination_tile\": \"Check coolant in all connected rooms\", \"confidence\": 0.85}", "done": true, "eval_duration": 120000000, "total_duration": 200000000}"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(response.as_bytes());
                        let _ = stream.flush();
                    }
                }
            }
        });

        let client = OllamaClient::new(base, 5);
        let config = OllamaModelConfig::fleet_default("test-fleet");
        let mut model = RealFleetModel::new(client, config);

        let readings = vec![
            test_reading("coolant", 95.0, 60.0, 90.0),
            test_reading("rpm", 4500.0, 1000.0, 3000.0),
        ];
        let result = model.analyze(&readings).await;

        shutdown.store(true, std::sync::atomic::Ordering::Relaxed);

        assert!(result.is_some(), "Expected a coordination tile");
        let (tile, confidence) = result.unwrap();
        assert_eq!(tile.tile_type, TileType::Coordination);
        assert!(confidence >= 0.6);
        assert_eq!(model.coordination_count, 1);
    }

    // ── OllamaClient Tests (no actual HTTP) ────────────────────────

    #[test]
    fn test_ollama_client_default() {
        let client = OllamaClient::default();
        // Should point to localhost:11434
        assert!(client.base_url.contains("localhost:11434"));
    }

    #[test]
    fn test_ollama_client_custom_url() {
        let client = OllamaClient::new("http://ollama.local:8080".to_string(), 60);
        assert_eq!(client.base_url, "http://ollama.local:8080");
        assert_eq!(client.timeout_secs, 60);
    }

    #[tokio::test]
    async fn test_ollama_client_server_offline() {
        // Connect to a port that's definitely not running
        let client = OllamaClient::new("http://127.0.0.1:19999".to_string(), 2);
        let result = client.generate("test-model", "ping", None).await;
        assert!(result.is_err(), "Should error when server is offline");
    }

    #[tokio::test]
    async fn test_list_models_server_offline() {
        let client = OllamaClient::new("http://127.0.0.1:19998".to_string(), 2);
        let result = client.list_models().await;
        assert!(result.is_err());
    }

    // ── OllamaModelConfig Tests ────────────────────────────────────

    #[test]
    fn test_nano_config_uses_correct_model() {
        let config = OllamaModelConfig::nano_default("liquid-350m");
        assert_eq!(config.model_name, "liquid-350m");
        assert!((config.confidence_threshold - 0.7).abs() < 0.01);
        assert_eq!(config.max_tokens, 64);
    }

    #[test]
    fn test_fleet_config_uses_correct_model() {
        let config = OllamaModelConfig::fleet_default("liquid-1.2b");
        assert_eq!(config.model_name, "liquid-1.2b");
        assert!((config.confidence_threshold - 0.6).abs() < 0.01);
        assert_eq!(config.max_tokens, 128);
    }

    // ── Error Display Tests ────────────────────────────────────────

    #[test]
    fn test_ollama_error_display() {
        let err = OllamaError::Parse("bad json".to_string());
        assert!(err.to_string().contains("bad json"));

        let err = OllamaError::Timeout;
        assert_eq!(err.to_string(), "Request timed out");

        let err = OllamaError::ModelNotAvailable("foo".to_string());
        assert!(err.to_string().contains("foo"));
    }
}
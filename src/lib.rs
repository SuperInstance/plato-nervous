//! Plato Nervous System — Room-Specific Model Distillation
//!
//! Each room starts with big LLM calls. As tiles accumulate, tiny distilled models
//! take over. Eventually each room trains its own LoRA — the irreducible intelligence
//! that can't be pure algorithm.
//!
//! Signal chain: Sensor → Deadband → Nano → LoRA → Fleet → Cloud
//! Each layer resolves what it can and passes the rest up.

pub mod ollama;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Core Types ────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    pub sensor_id: String,
    pub room_id: String,
    pub value: f64,
    pub unit: String,
    pub timestamp_ms: u64,
    pub normal_min: f64,
    pub normal_max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub id: Uuid,
    pub room_id: String,
    pub tile_type: TileType,
    pub content: String,
    pub confidence: f64,
    pub resolved_by: ResolutionLayer,
    pub timestamp_ms: u64,
    pub sensor_reading: Option<SensorReading>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum TileType {
    Status,
    Alert,
    Prediction,
    Anomaly,
    Coordination,
    Escalation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ResolutionLayer {
    /// Layer 0: Pure algorithmic (deadband, thresholds)
    Algorithmic,
    /// Layer 1: Nano-model (350M, anomaly detection)
    NanoModel,
    /// Layer 2: Room LoRA (350M + LoRA, room-specific reasoning)
    RoomLora,
    /// Layer 3: Fleet coordinator (1.2B, cross-room)
    FleetCoord,
    /// Layer 4: Cloud LLM (API call, novel situations)
    CloudEscalation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileExample {
    pub input: String,
    pub output: String,
    pub quality: f64,       // 0-1, was this tile useful?
    pub layer: ResolutionLayer,
    pub timestamp_ms: u64,
}

// ── Layer 0: Algorithmic Filters ─────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadbandFilter {
    pub deadband: f64,      // spectral gap width
    pub last_value: Option<f64>,
}

impl DeadbandFilter {
    pub fn new(deadband: f64) -> Self {
        Self { deadband, last_value: None }
    }

    /// Returns Some(tile) if the reading is within deadband (predictable = algorithmic)
    /// Returns None if the reading drifts outside deadband (needs model)
    pub fn check(&mut self, reading: &SensorReading) -> Option<Tile> {
        let in_range = reading.value >= reading.normal_min && reading.value <= reading.normal_max;

        match self.last_value {
            Some(prev) => {
                let drift = (reading.value - prev).abs();
                let in_deadband = drift <= self.deadband;

                if in_range && in_deadband {
                    // Fully predictable — algorithmic resolution
                    self.last_value = Some(reading.value);
                    Some(Tile {
                        id: Uuid::new_v4(),
                        room_id: reading.room_id.clone(),
                        tile_type: TileType::Status,
                        content: format!("{}: {:.1}{} (normal, drift {:.2})", 
                            reading.sensor_id, reading.value, reading.unit, drift),
                        confidence: 1.0,
                        resolved_by: ResolutionLayer::Algorithmic,
                        timestamp_ms: reading.timestamp_ms,
                        sensor_reading: Some(reading.clone()),
                    })
                } else {
                    // Drifted outside deadband or out of range — needs model
                    self.last_value = Some(reading.value);
                    None
                }
            }
            None => {
                self.last_value = Some(reading.value);
                if in_range {
                    Some(Tile {
                        id: Uuid::new_v4(),
                        room_id: reading.room_id.clone(),
                        tile_type: TileType::Status,
                        content: format!("{}: {:.1}{} (initial reading, normal)", 
                            reading.sensor_id, reading.value, reading.unit),
                        confidence: 1.0,
                        resolved_by: ResolutionLayer::Algorithmic,
                        timestamp_ms: reading.timestamp_ms,
                        sensor_reading: Some(reading.clone()),
                    })
                } else {
                    None
                }
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,
    pub condition: RuleCondition,
    pub tile_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    AboveThreshold { sensor_id: String, threshold: f64 },
    BelowThreshold { sensor_id: String, threshold: f64 },
    RateOfChange { sensor_id: String, max_delta_per_sec: f64 },
}

impl Rule {
    pub fn evaluate(&self, reading: &SensorReading) -> Option<Tile> {
        match &self.condition {
            RuleCondition::AboveThreshold { sensor_id, threshold } => {
                if reading.sensor_id == *sensor_id && reading.value > *threshold {
                    Some(Tile {
                        id: Uuid::new_v4(),
                        room_id: reading.room_id.clone(),
                        tile_type: TileType::Alert,
                        content: self.tile_content.clone(),
                        confidence: 1.0,
                        resolved_by: ResolutionLayer::Algorithmic,
                        timestamp_ms: reading.timestamp_ms,
                        sensor_reading: Some(reading.clone()),
                    })
                } else { None }
            }
            RuleCondition::BelowThreshold { sensor_id, threshold } => {
                if reading.sensor_id == *sensor_id && reading.value < *threshold {
                    Some(Tile {
                        id: Uuid::new_v4(),
                        room_id: reading.room_id.clone(),
                        tile_type: TileType::Alert,
                        content: self.tile_content.clone(),
                        confidence: 1.0,
                        resolved_by: ResolutionLayer::Algorithmic,
                        timestamp_ms: reading.timestamp_ms,
                        sensor_reading: Some(reading.clone()),
                    })
                } else { None }
            }
            RuleCondition::RateOfChange { .. } => None, // Needs history tracking
        }
    }
}

// ── Layer 1-2: Model Configuration ───────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_type: ModelType,
    pub model_path: Option<String>,    // Path to GGUF file
    pub endpoint: Option<String>,      // API endpoint
    pub max_tokens: usize,
    pub temperature: f64,
    pub confidence_threshold: f64,     // Below this, pass to next layer
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModelType {
    /// Liquid LFM2.5-350M — edge anomaly detection
    LiquidNano350M,
    /// Liquid LFM2.5-1.2B Instruct — room reasoning
    Liquid1_2BInstruct,
    /// Room-specific LoRA on 350M base
    RoomLora { base_model: String, lora_path: String, rank: usize },
    /// Fleet coordinator
    FleetCoordinator { model_path: String },
    /// Cloud API fallback
    CloudApi { provider: String, model: String },
}

// ── Layer 1: Nano Model (simulated for now) ──────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NanoModel {
    pub config: ModelConfig,
    pub prompt_template: String,
    /// Number of tiles this model has produced
    pub tiles_produced: usize,
    /// Running confidence average
    pub avg_confidence: f64,
}

impl NanoModel {
    pub fn new(config: ModelConfig, prompt_template: String) -> Self {
        Self { config, prompt_template, tiles_produced: 0, avg_confidence: 0.5 }
    }

    /// Process a reading through the nano model
    /// In production, this calls llama.cpp/ollama. Here we simulate.
    pub fn infer(&mut self, reading: &SensorReading) -> Option<(Tile, f64)> {
        let prompt = self.prompt_template
            .replace("{sensor_id}", &reading.sensor_id)
            .replace("{value}", &format!("{:.1}", reading.value))
            .replace("{unit}", &reading.unit)
            .replace("{normal_min}", &format!("{:.1}", reading.normal_min))
            .replace("{normal_max}", &format!("{:.1}", reading.normal_max));

        // Simulation: check if value is near the boundary of normal
        let range = reading.normal_max - reading.normal_min;
        let margin = range * 0.15; // 15% margin
        let near_boundary = reading.value < reading.normal_min + margin 
            || reading.value > reading.normal_max - margin;

        let (tile_type, confidence, content) = if !near_boundary {
            // Well within normal range — high confidence, no alert needed
            (TileType::Status, 0.95, 
             format!("{}: {:.1}{} — within normal range", reading.sensor_id, reading.value, reading.unit))
        } else if reading.value >= reading.normal_min && reading.value <= reading.normal_max {
            // Near boundary but still normal — moderate confidence
            (TileType::Status, 0.75,
             format!("{}: {:.1}{} — approaching boundary of normal range", 
                reading.sensor_id, reading.value, reading.unit))
        } else {
            // Outside normal — low confidence, should escalate
            return None;
        };

        if confidence >= self.config.confidence_threshold {
            self.tiles_produced += 1;
            self.avg_confidence = (self.avg_confidence * (self.tiles_produced - 1) as f64 
                + confidence) / self.tiles_produced as f64;
            
            Some((Tile {
                id: Uuid::new_v4(),
                room_id: reading.room_id.clone(),
                tile_type,
                content,
                confidence,
                resolved_by: ResolutionLayer::NanoModel,
                timestamp_ms: reading.timestamp_ms,
                sensor_reading: Some(reading.clone()),
            }, confidence))
        } else {
            None
        }
    }
}

// ── Signal Chain (the full nervous system) ────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomNervousSystem {
    pub room_id: String,
    pub room_name: String,
    
    // Layer 0
    pub deadband_filters: Vec<DeadbandFilter>,
    pub rules: Vec<Rule>,
    
    // Layer 1
    pub nano_model: Option<NanoModel>,
    
    // Layer 2 (placeholder — LoRA training not yet implemented)
    pub room_lora_trained: bool,
    pub room_lora_rank: usize,
    
    // Layer 3 (placeholder)
    pub fleet_model_available: bool,
    
    // Training data
    pub tile_buffer: Vec<TileExample>,
    pub max_tile_buffer: usize,
    
    // Statistics
    pub stats: NervousSystemStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NervousSystemStats {
    pub total_readings: u64,
    pub resolved_algorithmic: u64,
    pub resolved_nano: u64,
    pub resolved_lora: u64,
    pub resolved_fleet: u64,
    pub escalated_cloud: u64,
    pub tiles_produced: u64,
}

impl Default for NervousSystemStats {
    fn default() -> Self {
        Self {
            total_readings: 0, resolved_algorithmic: 0,
            resolved_nano: 0, resolved_lora: 0,
            resolved_fleet: 0, escalated_cloud: 0,
            tiles_produced: 0,
        }
    }
}

impl RoomNervousSystem {
    pub fn new(room_id: &str, room_name: &str) -> Self {
        Self {
            room_id: room_id.to_string(),
            room_name: room_name.to_string(),
            deadband_filters: Vec::new(),
            rules: Vec::new(),
            nano_model: None,
            room_lora_trained: false,
            room_lora_rank: 0,
            fleet_model_available: false,
            tile_buffer: Vec::new(),
            max_tile_buffer: 1000,
            stats: NervousSystemStats::default(),
        }
    }

    /// Add a deadband filter for a sensor
    pub fn with_deadband(mut self, deadband: f64) -> Self {
        self.deadband_filters.push(DeadbandFilter::new(deadband));
        self
    }

    /// Add an algorithmic rule
    pub fn with_rule(mut self, rule: Rule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Enable nano model (Layer 1)
    pub fn with_nano_model(mut self, config: ModelConfig, prompt_template: String) -> Self {
        self.nano_model = Some(NanoModel::new(config, prompt_template));
        self
    }

    /// Process a sensor reading through the full signal chain
    pub fn process(&mut self, reading: SensorReading) -> SignalResolution {
        self.stats.total_readings += 1;

        // Layer 0: Deadband filters
        for filter in &mut self.deadband_filters {
            if let Some(tile) = filter.check(&reading) {
                self.stats.resolved_algorithmic += 1;
                self.stats.tiles_produced += 1;
                self.record_tile(&tile, 1.0);
                return SignalResolution::Algorithmic(tile);
            }
        }

        // Layer 0: Rules
        for rule in &self.rules {
            if let Some(tile) = rule.evaluate(&reading) {
                self.stats.resolved_algorithmic += 1;
                self.stats.tiles_produced += 1;
                self.record_tile(&tile, 1.0);
                return SignalResolution::Algorithmic(tile);
            }
        }

        // Layer 1: Nano model
        if let Some(ref mut nano) = self.nano_model {
            if let Some((tile, confidence)) = nano.infer(&reading) {
                self.stats.resolved_nano += 1;
                self.stats.tiles_produced += 1;
                self.record_tile(&tile, confidence);
                return SignalResolution::NanoModel(tile, confidence);
            }
        }

        // Layer 4: Cloud escalation (no Layer 2/3 implemented yet)
        self.stats.escalated_cloud += 1;
        let tile = Tile {
            id: Uuid::new_v4(),
            room_id: reading.room_id.clone(),
            tile_type: TileType::Escalation,
            content: format!("ESCALATED: {}={:.1}{} — all local layers insufficient",
                reading.sensor_id, reading.value, reading.unit),
            confidence: 0.0,
            resolved_by: ResolutionLayer::CloudEscalation,
            timestamp_ms: reading.timestamp_ms,
            sensor_reading: Some(reading.clone()),
        };
        self.record_tile(&tile, 0.0);
        SignalResolution::Escalated(tile, "All local layers insufficient".into())
    }

    fn record_tile(&mut self, tile: &Tile, quality: f64) {
        let example = TileExample {
            input: tile.sensor_reading.as_ref()
                .map(|r| format!("{}={:.1}{}", r.sensor_id, r.value, r.unit))
                .unwrap_or_default(),
            output: tile.content.clone(),
            quality,
            layer: tile.resolved_by,
            timestamp_ms: tile.timestamp_ms,
        };
        self.tile_buffer.push(example);
        if self.tile_buffer.len() > self.max_tile_buffer {
            self.tile_buffer.remove(0);
        }
    }

    /// How autonomous is this room? (0.0 = fully cloud, 1.0 = fully local)
    pub fn autonomy_level(&self) -> f64 {
        if self.stats.total_readings == 0 { return 0.0; }
        let local = self.stats.resolved_algorithmic 
            + self.stats.resolved_nano 
            + self.stats.resolved_lora 
            + self.stats.resolved_fleet;
        local as f64 / self.stats.total_readings as f64
    }

    /// Get the resolution distribution
    pub fn resolution_distribution(&self) -> ResolutionDistribution {
        let total = self.stats.total_readings.max(1) as f64;
        ResolutionDistribution {
            algorithmic_pct: self.stats.resolved_algorithmic as f64 / total * 100.0,
            nano_pct: self.stats.resolved_nano as f64 / total * 100.0,
            lora_pct: self.stats.resolved_lora as f64 / total * 100.0,
            fleet_pct: self.stats.resolved_fleet as f64 / total * 100.0,
            cloud_pct: self.stats.escalated_cloud as f64 / total * 100.0,
            autonomy: self.autonomy_level(),
        }
    }

    /// Is the room ready for LoRA training?
    pub fn ready_for_lora(&self) -> bool {
        self.tile_buffer.len() >= 100 
            && self.tile_buffer.iter().filter(|t| t.quality > 0.7).count() >= 50
    }

    /// Estimate cloud call reduction after distillation
    pub fn estimate_reduction(&self) -> f64 {
        let current_cloud_pct = self.stats.escalated_cloud as f64 
            / self.stats.total_readings.max(1) as f64;
        // LoRA typically absorbs 80% of what nano can't handle
        let after_nano = current_cloud_pct * 0.2;
        let after_lora = after_nano * 0.2;
        after_lora
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionDistribution {
    pub algorithmic_pct: f64,
    pub nano_pct: f64,
    pub lora_pct: f64,
    pub fleet_pct: f64,
    pub cloud_pct: f64,
    pub autonomy: f64,
}

pub enum SignalResolution {
    Algorithmic(Tile),
    NanoModel(Tile, f64),
    RoomLora(Tile, f64),
    FleetCoord(Tile, f64),
    Escalated(Tile, String),
}

// ── Distillation Pipeline ─────────────────────────────────────────────
// Each layer is both a RESOLVER and a DISTILLER. When a lower layer resolves
// a reading, it produces a training example for the layer above. When a
// higher layer resolves what a lower layer couldn't, its response becomes
// training data for the lower layer's LoRA.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationRecord {
    pub input_hash: u64,
    pub layer_resolved: ResolutionLayer,
    pub confidence: f64,
    /// Time taken to resolve (ms)
    pub latency_ms: u64,
    /// Was this resolved correctly? (determined by downstream verification)
    pub verified_correct: Option<bool>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationStats {
    /// How many tiles have been used for LoRA training so far
    pub total_tiles_used: usize,
    /// Accuracy of the nano model BEFORE last distillation
    pub pre_distillation_accuracy: f64,
    /// Accuracy AFTER last distillation
    pub post_distillation_accuracy: f64,
    /// Number of distillation cycles completed
    pub distillation_cycles: usize,
    /// Conservation ratio at each layer transition
    pub cr_l0_to_l1: f64,
    pub cr_l1_to_l2: f64,
    pub cr_l2_to_l3: f64,
    pub cr_l3_to_l4: f64,
    /// Cloud call reduction after each cycle
    pub cloud_reduction_pct: f64,
}

impl Default for DistillationStats {
    fn default() -> Self {
        Self {
            total_tiles_used: 0,
            pre_distillation_accuracy: 0.0,
            post_distillation_accuracy: 0.0,
            distillation_cycles: 0,
            cr_l0_to_l1: 0.99, // Algorithmic is near-perfect
            cr_l1_to_l2: 0.0,  // No LoRA yet
            cr_l2_to_l3: 0.0,
            cr_l3_to_l4: 0.0,
            cloud_reduction_pct: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationConfig {
    /// Minimum tiles before first distillation
    pub min_tiles_for_lora: usize,
    /// Minimum HIGH-QUALITY tiles (quality > 0.7) needed
    pub min_high_quality_tiles: usize,
    /// LoRA rank for room-specific adapter
    pub lora_rank: usize,
    /// How often to re-distill (in readings processed)
    pub redistillation_interval: usize,
    /// CR threshold below which re-distillation triggers
    pub cr_redistillation_threshold: f64,
    /// Maximum LoRA training epochs
    pub max_epochs: usize,
}

impl Default for DistillationConfig {
    fn default() -> Self {
        Self {
            min_tiles_for_lora: 100,
            min_high_quality_tiles: 50,
            lora_rank: 8,
            redistillation_interval: 1000,
            cr_redistillation_threshold: 0.85,
            max_epochs: 10,
        }
    }
}

// ── JEPA-like Room Perception (The Irreducible Core) ──────────────────
// After weeks of operation, each room develops a self-model — a compressed
// representation of its own state that captures holistic patterns no single
// sensor reading can express. This is the JEPA nano-model.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomStateVector {
    pub room_id: String,
    /// Compressed state of the room (16 dimensions)
    /// Each dimension captures a holistic aspect:
    ///   [0] - overall health (0 = critical, 1 = perfect)
    ///   [1] - thermal trend (negative = cooling, positive = heating)
    ///   [2] - vibration signature (higher = more vibration)
    ///   [3] - stress level (cognitive load on the room)
    ///   [4] - drift rate (how fast things are changing)
    ///   [5-7] - cross-sensor correlations (RPM↔coolant, RPM↔oil, coolant↔oil)
    ///   [8-11] - temporal patterns (hourly, daily, weekly, seasonal)
    ///   [12-15] - reserved for room-specific dimensions
    pub state: [f32; 16],
    /// Confidence in the state vector (how well the JEPA model knows this state)
    pub confidence: f64,
    pub timestamp_ms: u64,
}

impl RoomStateVector {
    pub fn health(&self) -> f32 { self.state[0] }
    pub fn thermal_trend(&self) -> f32 { self.state[1] }
    pub fn vibration(&self) -> f32 { self.state[2] }
    pub fn stress(&self) -> f32 { self.state[3] }
    pub fn drift_rate(&self) -> f32 { self.state[4] }
    
    /// Is the room in a known-good state? (health > 0.7, low stress)
    pub fn is_healthy(&self) -> bool {
        self.state[0] > 0.7 && self.state[3] < 0.3
    }
    
    /// Is the room in an anomalous state? (low health OR high stress)
    pub fn is_anomalous(&self) -> bool {
        self.state[0] < 0.3 || self.state[3] > 0.7
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaNanoConfig {
    /// Input embedding dimension (from the 350M model's output)
    pub input_dim: usize,
    /// Room state vector dimension
    pub state_dim: usize,
    /// Model size in parameters (1M-10M for edge deployment)
    pub param_count: usize,
    /// Prediction horizon (how far ahead the JEPA model predicts)
    pub prediction_horizon_ms: u64,
}

impl Default for JepaNanoConfig {
    fn default() -> Self {
        Self {
            input_dim: 384,   // Standard small embedding dim
            state_dim: 16,    // Room state vector
            param_count: 2_000_000, // 2M params — fits in ~4MB
            prediction_horizon_ms: 60_000, // Predict 1 minute ahead
        }
    }
}

/// The JEPA (Joint Embedding Predictive Architecture) nano-model
/// This is the irreducible core — the room's self-model.
/// It doesn't process language. It processes embeddings.
/// Its job: predict the NEXT room state from the current state.
/// When predictions diverge from reality, that's anomaly detection
/// that no threshold or rule could ever catch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JepaNano {
    pub config: JepaNanoConfig,
    /// The learned state transition matrix (state_dim × state_dim)
    /// In a real implementation, this would be a small neural net.
    /// Here it's a simple linear model for demonstration.
    pub transition_weights: Vec<Vec<f32>>,
    /// Running average prediction error (used for anomaly detection)
    pub avg_prediction_error: f64,
    /// Number of states processed
    pub states_processed: u64,
    /// Last predicted state
    pub last_prediction: Option<RoomStateVector>,
}

impl JepaNano {
    pub fn new(config: JepaNanoConfig) -> Self {
        let dim = config.state_dim;
        // Initialize with identity-like weights (predict current = next)
        let mut weights = vec![vec![0.0f32; dim]; dim];
        for i in 0..dim {
            weights[i][i] = 0.9; // Slight decay toward zero
        }
        Self {
            config,
            transition_weights: weights,
            avg_prediction_error: 0.0,
            states_processed: 0,
            last_prediction: None,
        }
    }
    
    /// Predict the next room state from the current state
    pub fn predict(&self, current: &RoomStateVector) -> RoomStateVector {
        let dim = self.config.state_dim;
        let mut next_state = [0.0f32; 16];
        
        for i in 0..dim.min(16) {
            let mut val = 0.0f32;
            for j in 0..dim.min(16) {
                val += self.transition_weights[i][j] * current.state[j];
            }
            next_state[i] = val;
        }
        
        RoomStateVector {
            room_id: current.room_id.clone(),
            state: next_state,
            confidence: current.confidence * 0.95, // Confidence decays
            timestamp_ms: current.timestamp_ms + self.config.prediction_horizon_ms,
        }
    }
    
    /// Update the model with an actual observation
    /// Returns the prediction error (how surprised the model was)
    pub fn update(&mut self, actual: &RoomStateVector) -> f64 {
        let error = if let Some(ref predicted) = self.last_prediction {
            let mut total_error = 0.0f64;
            for i in 0..16 {
                total_error += (predicted.state[i] - actual.state[i]).powi(2) as f64;
            }
            (total_error / 16.0).sqrt() // RMSE
        } else {
            0.0
        };
        
        // Update running average
        self.states_processed += 1;
        let alpha = 1.0 / self.states_processed.min(100) as f64;
        self.avg_prediction_error = self.avg_prediction_error * (1.0 - alpha) + error * alpha;
        
        // Online learning: nudge weights toward the actual transition
        // (In production, this would be a proper gradient step)
        if let Some(ref predicted) = self.last_prediction {
            let lr = 0.01; // Learning rate
            for i in 0..self.config.state_dim.min(16) {
                let delta = actual.state[i] - predicted.state[i];
                for j in 0..self.config.state_dim.min(16) {
                    self.transition_weights[i][j] += lr * delta * actual.state[j];
                }
            }
        }
        
        // Set up next prediction
        self.last_prediction = Some(self.predict(actual));
        
        error
    }
    
    /// Is the current prediction error anomalously high?
    /// This is the JEPA anomaly detection — when the model is surprised.
    pub fn is_surprised(&self, error: f64) -> bool {
        if self.states_processed < 10 { return false; }
        error > self.avg_prediction_error * 3.0 // 3σ threshold
    }
    
    /// How "well-trained" is this model? (0 = newborn, 1 = fully trained)
    pub fn maturity(&self) -> f64 {
        (self.states_processed as f64 / 10000.0).min(1.0)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_reading(sensor_id: &str, value: f64, min: f64, max: f64) -> SensorReading {
        SensorReading {
            sensor_id: sensor_id.to_string(),
            room_id: "engine-room".to_string(),
            value, unit: "units".to_string(),
            timestamp_ms: 1000, normal_min: min, normal_max: max,
        }
    }

    #[test]
    fn test_deadband_normal_reading() {
        let mut filter = DeadbandFilter::new(5.0);
        let reading = make_reading("rpm", 1450.0, 1400.0, 1500.0);
        let result = filter.check(&reading);
        assert!(result.is_some());
        assert_eq!(result.unwrap().resolved_by, ResolutionLayer::Algorithmic);
    }

    #[test]
    fn test_deadband_small_drift() {
        let mut filter = DeadbandFilter::new(5.0);
        let r1 = make_reading("rpm", 1450.0, 1400.0, 1500.0);
        filter.check(&r1);
        let r2 = make_reading("rpm", 1453.0, 1400.0, 1500.0);
        let result = filter.check(&r2);
        assert!(result.is_some()); // 3.0 drift < 5.0 deadband
    }

    #[test]
    fn test_deadband_large_drift() {
        let mut filter = DeadbandFilter::new(5.0);
        let r1 = make_reading("rpm", 1450.0, 1400.0, 1500.0);
        filter.check(&r1);
        let r2 = make_reading("rpm", 1460.0, 1400.0, 1500.0);
        let result = filter.check(&r2);
        assert!(result.is_none()); // 10.0 drift > 5.0 deadband → needs model
    }

    #[test]
    fn test_deadband_out_of_range() {
        let mut filter = DeadbandFilter::new(5.0);
        let reading = make_reading("coolant", 228.0, 140.0, 210.0);
        let result = filter.check(&reading);
        assert!(result.is_none()); // Out of normal range → needs model
    }

    #[test]
    fn test_rule_above_threshold() {
        let rule = Rule {
            name: "high_coolant".to_string(),
            condition: RuleCondition::AboveThreshold {
                sensor_id: "coolant".to_string(), threshold: 210.0,
            },
            tile_content: "Coolant above 210F".to_string(),
        };
        let reading = make_reading("coolant", 215.0, 140.0, 210.0);
        let result = rule.evaluate(&reading);
        assert!(result.is_some());
        assert_eq!(result.unwrap().tile_type, TileType::Alert);
    }

    #[test]
    fn test_rule_below_threshold() {
        let rule = Rule {
            name: "low_oil".to_string(),
            condition: RuleCondition::BelowThreshold {
                sensor_id: "oil".to_string(), threshold: 35.0,
            },
            tile_content: "Oil below 35 PSI".to_string(),
        };
        let reading = make_reading("oil", 28.0, 35.0, 80.0);
        let result = rule.evaluate(&reading);
        assert!(result.is_some());
    }

    #[test]
    fn test_rule_no_match() {
        let rule = Rule {
            name: "high_coolant".to_string(),
            condition: RuleCondition::AboveThreshold {
                sensor_id: "coolant".to_string(), threshold: 210.0,
            },
            tile_content: "Coolant above 210F".to_string(),
        };
        let reading = make_reading("coolant", 195.0, 140.0, 210.0);
        let result = rule.evaluate(&reading);
        assert!(result.is_none());
    }

    #[test]
    fn test_nano_model_normal_reading() {
        let config = ModelConfig {
            model_type: ModelType::LiquidNano350M,
            model_path: None, endpoint: None,
            max_tokens: 32, temperature: 0.0,
            confidence_threshold: 0.7,
        };
        let mut nano = NanoModel::new(config, 
            "{sensor_id}={value}{unit} normal:{normal_min}-{normal_max}".to_string());
        
        let reading = make_reading("rpm", 1450.0, 1400.0, 1500.0);
        let result = nano.infer(&reading);
        assert!(result.is_some());
        let (_, conf) = result.unwrap();
        assert!(conf >= 0.7);
    }

    #[test]
    fn test_nano_model_boundary_reading() {
        let config = ModelConfig {
            model_type: ModelType::LiquidNano350M,
            model_path: None, endpoint: None,
            max_tokens: 32, temperature: 0.0,
            confidence_threshold: 0.7,
        };
        let mut nano = NanoModel::new(config, "".to_string());
        
        // Value near boundary (within 15% margin)
        let reading = make_reading("rpm", 1493.0, 1400.0, 1500.0);
        let result = nano.infer(&reading);
        assert!(result.is_some());
        let (_, conf) = result.unwrap();
        assert!(conf < 0.95); // Should be less confident near boundary
    }

    #[test]
    fn test_nano_model_out_of_range() {
        let config = ModelConfig {
            model_type: ModelType::LiquidNano350M,
            model_path: None, endpoint: None,
            max_tokens: 32, temperature: 0.0,
            confidence_threshold: 0.7,
        };
        let mut nano = NanoModel::new(config, "".to_string());
        
        let reading = make_reading("rpm", 1650.0, 1400.0, 1500.0);
        let result = nano.infer(&reading);
        assert!(result.is_none()); // Out of range → can't handle → pass up
    }

    #[test]
    fn test_full_signal_chain_mostly_algorithmic() {
        let mut ns = RoomNervousSystem::new("engine-room", "Engine Room");
        ns.deadband_filters.push(DeadbandFilter::new(10.0));
        ns.rules.push(Rule {
            name: "high_coolant".to_string(),
            condition: RuleCondition::AboveThreshold {
                sensor_id: "coolant".to_string(), threshold: 210.0,
            },
            tile_content: "Coolant above 210F!".to_string(),
        });

        // Feed 100 normal readings
        for i in 0..100 {
            let reading = SensorReading {
                sensor_id: "rpm".to_string(),
                room_id: "engine-room".to_string(),
                value: 1450.0 + (i as f64 * 0.1).sin() * 5.0, // Gentle oscillation
                unit: "rpm".to_string(),
                timestamp_ms: i * 1000,
                normal_min: 1400.0, normal_max: 1500.0,
            };
            ns.process(reading.clone());
        }

        // Most should be resolved algorithmically
        assert!(ns.autonomy_level() > 0.9);
        assert_eq!(ns.stats.escalated_cloud, 0);
    }

    #[test]
    fn test_full_signal_chain_with_anomaly() {
        let mut ns = RoomNervousSystem::new("engine-room", "Engine Room");
        ns.deadband_filters.push(DeadbandFilter::new(10.0));
        
        // Feed normal, then anomaly
        for i in 0..10 {
            let reading = SensorReading {
                sensor_id: "coolant".to_string(),
                room_id: "engine-room".to_string(),
                value: 195.0,
                unit: "F".to_string(),
                timestamp_ms: i * 1000,
                normal_min: 140.0, normal_max: 210.0,
            };
            ns.process(reading.clone());
        }

        // Now send an anomaly
        let anomaly = SensorReading {
            sensor_id: "coolant".to_string(),
            room_id: "engine-room".to_string(),
            value: 228.0, // Way above normal max
            unit: "F".to_string(),
            timestamp_ms: 10000,
            normal_min: 140.0, normal_max: 210.0,
        };
        let result = ns.process(anomaly);
        
        // Should be escalated (no rules, no nano model configured)
        match result {
            SignalResolution::Escalated(tile, _) => {
                assert_eq!(tile.tile_type, TileType::Escalation);
            }
            _ => panic!("Expected escalation for out-of-range reading"),
        }
    }

    #[test]
    fn test_tile_buffer_fills() {
        let mut ns = RoomNervousSystem::new("room", "Test Room");
        ns.max_tile_buffer = 10;
        ns.deadband_filters.push(DeadbandFilter::new(100.0));

        for i in 0..15 {
            let reading = make_reading("temp", 20.0 + i as f64, 0.0, 100.0);
            ns.process(reading.clone());
        }

        assert_eq!(ns.tile_buffer.len(), 10); // Capped at max
    }

    #[test]
    fn test_ready_for_lora() {
        let mut ns = RoomNervousSystem::new("room", "Test Room");
        ns.max_tile_buffer = 200;
        ns.deadband_filters.push(DeadbandFilter::new(100.0));

        // Not ready with few tiles
        for i in 0..50 {
            let reading = make_reading("temp", 20.0, 0.0, 100.0);
            ns.process(reading.clone());
        }
        assert!(!ns.ready_for_lora());

        // Ready after 100+ tiles with good quality
        for i in 0..100 {
            let reading = make_reading("temp", 20.0 + i as f64 * 0.01, 0.0, 100.0);
            ns.process(reading.clone());
        }
        assert!(ns.ready_for_lora());
    }

    #[test]
    fn test_autonomy_level_calculation() {
        let mut ns = RoomNervousSystem::new("room", "Test Room");
        ns.deadband_filters.push(DeadbandFilter::new(100.0));

        // 10 normal readings
        for _ in 0..10 {
            ns.process(make_reading("x", 50.0, 0.0, 100.0));
        }
        assert_eq!(ns.autonomy_level(), 1.0); // All algorithmic

        // 1 anomaly (out of range)
        ns.process(make_reading("x", 150.0, 0.0, 100.0));
        assert!(ns.autonomy_level() < 1.0);
        assert!(ns.autonomy_level() > 0.9); // 10/11 resolved locally
    }

    #[test]
    fn test_resolution_distribution() {
        let mut ns = RoomNervousSystem::new("room", "Test Room");
        ns.deadband_filters.push(DeadbandFilter::new(100.0));
        
        for _ in 0..8 { ns.process(make_reading("x", 50.0, 0.0, 100.0)); }
        for _ in 0..2 { ns.process(make_reading("x", 150.0, 0.0, 100.0)); } // escalated
        
        let dist = ns.resolution_distribution();
        assert!((dist.algorithmic_pct - 80.0).abs() < 1.0);
        assert!((dist.cloud_pct - 20.0).abs() < 1.0);
        assert!((dist.autonomy - 0.8).abs() < 0.01);
    }

    // ── Distillation Pipeline Tests ────────────────────────────────

    #[test]
    fn test_distillation_config_defaults() {
        let config = DistillationConfig::default();
        assert_eq!(config.min_tiles_for_lora, 100);
        assert_eq!(config.lora_rank, 8);
        assert!(config.cr_redistillation_threshold > 0.0);
    }

    #[test]
    fn test_distillation_stats_defaults() {
        let stats = DistillationStats::default();
        assert_eq!(stats.distillation_cycles, 0);
        assert!(stats.cr_l0_to_l1 > 0.9); // Algorithmic layer is near-perfect
    }

    // ── JEPA Nano-Model Tests ──────────────────────────────────────

    fn make_state(room_id: &str, health: f32, thermal: f32, stress: f32) -> RoomStateVector {
        let mut state = [0.0f32; 16];
        state[0] = health;
        state[1] = thermal;
        state[3] = stress;
        RoomStateVector {
            room_id: room_id.to_string(),
            state, confidence: 0.9, timestamp_ms: 1000,
        }
    }

    #[test]
    fn test_jepa_nano_creation() {
        let jepa = JepaNano::new(JepaNanoConfig::default());
        assert_eq!(jepa.states_processed, 0);
        assert_eq!(jepa.avg_prediction_error, 0.0);
        assert!(jepa.last_prediction.is_none());
    }

    #[test]
    fn test_jepa_prediction() {
        let jepa = JepaNano::new(JepaNanoConfig::default());
        let current = make_state("engine-room", 0.8, 0.1, 0.2);
        let predicted = jepa.predict(&current);
        
        // Diagonal weights are 0.9, so prediction should be close to current
        assert!((predicted.state[0] - 0.8 * 0.9).abs() < 0.01);
        assert!((predicted.state[1] - 0.1 * 0.9).abs() < 0.01);
    }

    #[test]
    fn test_jepa_learning() {
        let mut jepa = JepaNano::new(JepaNanoConfig::default());
        
        // Feed stable states — model should learn the pattern
        for i in 0..50 {
            let state = make_state("room", 0.8, 0.1 + i as f32 * 0.001, 0.2);
            jepa.update(&state);
        }
        
        assert!(jepa.states_processed == 50);
        assert!(jepa.avg_prediction_error < 1.0); // Should be learning
        assert!(jepa.last_prediction.is_some());
    }

    #[test]
    fn test_jepa_anomaly_detection() {
        let mut jepa = JepaNano::new(JepaNanoConfig::default());
        
        // Train on stable states
        for _ in 0..100 {
            let state = make_state("room", 0.8, 0.1, 0.2);
            jepa.update(&state);
        }
        
        // Normal reading — should not be surprised
        let normal = make_state("room", 0.8, 0.1, 0.2);
        let normal_error = jepa.update(&normal);
        assert!(!jepa.is_surprised(normal_error));
        
        // Anomalous reading — should be surprised
        let anomaly = make_state("room", 0.1, 0.9, 0.95);
        let anomaly_error = jepa.update(&anomaly);
        assert!(jepa.is_surprised(anomaly_error));
    }

    #[test]
    fn test_jepa_maturity() {
        let mut jepa = JepaNano::new(JepaNanoConfig::default());
        assert_eq!(jepa.maturity(), 0.0); // Newborn
        
        for _ in 0..5000 {
            let state = make_state("room", 0.8, 0.1, 0.2);
            jepa.update(&state);
        }
        assert!(jepa.maturity() > 0.4);
        assert!(jepa.maturity() < 1.0);
        
        for _ in 0..5000 {
            let state = make_state("room", 0.8, 0.1, 0.2);
            jepa.update(&state);
        }
        assert!((jepa.maturity() - 1.0).abs() < 0.01); // Fully mature
    }

    #[test]
    fn test_room_state_vector_accessors() {
        let mut sv = make_state("room", 0.8, 0.3, 0.2);
        sv.state[2] = 0.4; // vibration
        sv.state[4] = 0.1; // drift rate
        
        assert!((sv.health() - 0.8).abs() < 0.01);
        assert!((sv.thermal_trend() - 0.3).abs() < 0.01);
        assert!((sv.vibration() - 0.4).abs() < 0.01);
        assert!((sv.stress() - 0.2).abs() < 0.01);
        assert!((sv.drift_rate() - 0.1).abs() < 0.01);
        assert!(sv.is_healthy());
        assert!(!sv.is_anomalous());
    }

    #[test]
    fn test_room_state_vector_anomalous() {
        let sv = make_state("room", 0.1, 0.3, 0.9); // Low health, high stress
        assert!(!sv.is_healthy());
        assert!(sv.is_anomalous());
    }

    #[test]
    fn test_full_signal_chain_with_jepa() {
        // Simulate the full lifecycle: deadband → nano → cloud → JEPA
        let mut ns = RoomNervousSystem::new("engine-room", "Engine Room");
        ns.deadband_filters.push(DeadbandFilter::new(10.0));
        
        let mut jepa = JepaNano::new(JepaNanoConfig::default());
        
        // Phase 1: Normal operation — deadband catches most, JEPA learns the pattern
        for i in 0..200 {
            let reading = SensorReading {
                sensor_id: "rpm".to_string(),
                room_id: "engine-room".to_string(),
                value: 1450.0 + (i as f64 * 0.1).sin() * 5.0,
                unit: "rpm".to_string(),
                timestamp_ms: i * 1000,
                normal_min: 1400.0, normal_max: 1500.0,
            };
            ns.process(reading.clone());
            
            // Feed to JEPA as well
            let mut state = [0.0f32; 16];
            state[0] = 0.85; // healthy
            state[1] = (reading.value as f32 - 1450.0) / 50.0; // normalized thermal
            jepa.update(&RoomStateVector {
                room_id: "engine-room".to_string(),
                state, confidence: 0.9, timestamp_ms: reading.timestamp_ms,
            });
        }
        
        // Most resolved by deadband, JEPA has learned the pattern
        assert!(ns.autonomy_level() > 0.9);
        assert!(jepa.maturity() > 0.01);
        assert!(jepa.avg_prediction_error < 1.0);
        
        // Phase 2: Anomaly — JEPA should be surprised
        let mut anomaly_state = [0.0f32; 16];
        anomaly_state[0] = 0.2; // Low health
        anomaly_state[1] = 0.8; // High thermal trend
        anomaly_state[3] = 0.9; // High stress
        let anomaly_error = jepa.update(&RoomStateVector {
            room_id: "engine-room".to_string(),
            state: anomaly_state, confidence: 0.5, timestamp_ms: 200000,
        });
        
        // JEPA should be surprised by the anomaly
        assert!(jepa.is_surprised(anomaly_error));
    }
}

//! Plato Nervous System — Room-Specific Model Distillation
//!
//! Each room starts with big LLM calls. As tiles accumulate, tiny distilled models
//! take over. Eventually each room trains its own LoRA — the irreducible intelligence
//! that can't be pure algorithm.
//!
//! Signal chain: Sensor → Deadband → Nano → LoRA → Fleet → Cloud
//! Each layer resolves what it can and passes the rest up.

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
            ns.process(reading);
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
            ns.process(reading);
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
            ns.process(reading);
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
            ns.process(reading);
        }
        assert!(!ns.ready_for_lora());

        // Ready after 100+ tiles with good quality
        for i in 0..100 {
            let reading = make_reading("temp", 20.0 + i as f64 * 0.01, 0.0, 100.0);
            ns.process(reading);
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
}

use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};

// shared structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VehicleData {
    pub id: u64,
    pub waiting_time: u64,
    pub accident_timestamp: Option<u64>,
    pub severity: i8,
    pub current_lane: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficData {
    pub lane_occupancy: HashMap<String, f64>,
    pub accident_lanes: HashSet<String>,
    pub intersection_congestion: HashMap<String, f64>,
    pub intersection_waiting_time: HashMap<String, f64>,
    pub vehicle_data: Vec<VehicleData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficUpdate {
    pub current_data: TrafficData,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccidentInfo {
    pub vehicle_id: u64,
    pub accident_timestamp: u64,
    pub severity: i8,
    pub current_lane: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficEvent {
    pub timestamp: u64,
    pub average_vehicle_delay: f64,
    pub total_accidents: usize,
    pub accident_details: Vec<AccidentInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionAlert {
    pub timestamp: u64,
    pub intersection: Option<String>,
    pub message: String,
    pub congestion_perc: f64,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightAdjustment {
    pub timestamp: u64,
    pub intersection_id: String,
    pub add_seconds_green: u32,
}

// shared functions
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

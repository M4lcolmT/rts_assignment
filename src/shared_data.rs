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

// shared functions
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

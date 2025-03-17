// src/shared_data.rs

use crate::simulation_engine::intersections::IntersectionId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Core traffic data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficData {
    pub total_vehicles: usize,
    pub lane_occupancy: HashMap<String, f64>,
    pub accident_lanes: HashSet<String>,
    pub intersection_congestion: HashMap<IntersectionId, f64>,
    pub intersection_waiting_time: HashMap<IntersectionId, f64>,
}

/// A complete traffic update (current + predicted traffic data, timestamp)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficUpdate {
    pub current_data: TrafficData,
    pub predicted_data: TrafficData,
    pub timestamp: u64,
}

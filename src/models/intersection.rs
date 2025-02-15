use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intersection {
    pub id: u32,
    pub connected_intersections: Vec<u32>,
    pub current_vehicles: Vec<Uuid>,
    pub is_traffic_light_green: bool,
    pub max_capacity: u32,
}

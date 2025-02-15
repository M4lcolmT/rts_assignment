use crate::models::vehicle::Vehicle;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimulationMessage {
    VehicleSpawned(Vehicle),
    VehicleMoved {
        vehicle_id: Uuid,
        from: u32,
        to: u32,
    },
    TrafficLightChanged {
        intersection_id: u32,
        is_green: bool,
    },
    IntersectionCongested {
        intersection_id: u32,
        load: f32,
    },
    SimulationTick(f64),
}

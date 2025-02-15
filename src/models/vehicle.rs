use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VehicleType {
    Car(u32),
    Bus(u32),
    Emergency(u32),
    Truck(u32),
}

impl std::fmt::Display for VehicleType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VehicleType::Car(n) => write!(f, "Car {}", n),
            VehicleType::Bus(n) => write!(f, "Bus {}", n),
            VehicleType::Emergency(n) => write!(f, "Emergency {}", n),
            VehicleType::Truck(n) => write!(f, "Truck {}", n),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: Uuid,
    pub vehicle_type: VehicleType,
    pub current_intersection: Option<u32>,
    pub route: Vec<u32>,
    pub entry_time: DateTime<Local>,
    pub current_speed: f32,
    pub waiting_time: f32,
}

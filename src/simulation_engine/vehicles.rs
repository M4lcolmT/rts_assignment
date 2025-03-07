use crate::simulation_engine::intersections::IntersectionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Car,
    Bus,
    Truck,
    EmergencyVan,
}

#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: u64,
    pub vehicle_type: VehicleType,
    pub entry_point: IntersectionId,
    pub exit_point: IntersectionId,
    pub speed: f64,
    pub length: f64,
    pub is_emergency: bool,
    pub rerouted: bool,
    pub is_in_lane: bool,
    pub is_accident: bool,
    pub severity: i8,
    pub current_lane: String,
    // TODO: pub waiting_time: Option<u64>
    pub accident_timestamp: Option<u64>,
}

impl Vehicle {
    /// Creates a new vehicle with predefined lengths based on type.
    pub fn new(
        id: u64,
        vehicle_type: VehicleType,
        entry_point: IntersectionId,
        exit_point: IntersectionId,
        speed: f64,
    ) -> Self {
        let (length, is_emergency) = match vehicle_type {
            VehicleType::Car => (4.5, false),
            VehicleType::Bus => (12.0, false),
            VehicleType::Truck => (16.0, false),
            VehicleType::EmergencyVan => (5.5, true),
        };

        Self {
            id,
            vehicle_type,
            entry_point,
            exit_point,
            speed,
            length,
            is_emergency,
            rerouted: false,
            is_in_lane: false,
            is_accident: false,
            severity: 0,
            current_lane: "".to_string(),
            accident_timestamp: None,
        }
    }
}

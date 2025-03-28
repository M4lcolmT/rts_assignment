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
    pub rerouted: bool,
    pub is_in_lane: bool,
    pub is_accident: bool,
    pub severity: i8,
    pub current_lane: String,
    pub accident_timestamp: Option<u64>,
    // Accumulated waiting time (in seconds)
    pub waiting_time: u64,
    // When the vehicle started waiting (None if not waiting)
    pub waiting_start: Option<u64>,
}

impl Vehicle {
    /// Creates a new vehicle with predefined length based on type.
    pub fn new(
        id: u64,
        vehicle_type: VehicleType,
        entry_point: IntersectionId,
        exit_point: IntersectionId,
        speed: f64,
    ) -> Self {
        let length = match vehicle_type {
            VehicleType::Car => 2.0,
            VehicleType::Bus => 6.0,
            VehicleType::Truck => 4.0,
            VehicleType::EmergencyVan => 3.0,
        };

        Self {
            id,
            vehicle_type,
            entry_point,
            exit_point,
            speed,
            length,
            rerouted: false,
            is_in_lane: false,
            is_accident: false,
            severity: 0,
            current_lane: "".to_string(),
            accident_timestamp: None,
            waiting_time: 0,
            waiting_start: None,
        }
    }

    /// Returns true if the vehicle is an emergency vehicle.
    pub fn is_emergency(&self) -> bool {
        self.vehicle_type == VehicleType::EmergencyVan
    }
}

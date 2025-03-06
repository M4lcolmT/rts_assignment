// vehicles.rs
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
    pub waiting_logged: bool,   // already used for waiting message printing
    pub added_to_lane: bool,    // used to add occupancy only once while waiting
    pub accident_handled: bool, // NEW: marks if this vehicle has been processed for the current accident
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
            waiting_logged: false,
            added_to_lane: false,
            accident_handled: false, // Initialize as not processed
        }
    }
}

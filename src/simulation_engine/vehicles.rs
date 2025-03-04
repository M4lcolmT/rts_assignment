use crate::simulation_engine::intersections::IntersectionId;

/// Different types of vehicles in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Car,
    Bus,
    Truck,
    EmergencyVan,
}

/// Represents a vehicle traveling through the grid.
#[derive(Debug, Clone)]
pub struct Vehicle {
    pub id: u64,
    pub vehicle_type: VehicleType,
    pub entry_point: IntersectionId,
    pub exit_point: IntersectionId,
    pub speed: f64,
    pub length: f64,
    /// Priority (e.g., 0 for normal, higher for emergency vehicles).
    // pub priority: u8,
    // TODO: maybe add waiting time
    // TODO: maybe add re-route state flag to not allow re-routing for the vehicle again until it reaches the next intersection or makes noticeable progress.
    pub is_emergency: bool,
}

impl Vehicle {
    /// Creates a new vehicle with predefined lengths based on type.
    pub fn new(
        id: u64,
        vehicle_type: VehicleType,
        entry_point: IntersectionId,
        exit_point: IntersectionId,
        speed: f64,
        // priority: u8,
    ) -> Self {
        let (length, is_emergency) = match vehicle_type {
            VehicleType::Car => (4.5, false),
            VehicleType::Bus => (12.0, false),
            VehicleType::Truck => (16.0, false),
            VehicleType::EmergencyVan => (5.5, true), // Emergency vehicle flag
        };

        Self {
            id,
            vehicle_type,
            entry_point,
            exit_point,
            speed,
            length,
            // priority,
            is_emergency,
        }
    }
}

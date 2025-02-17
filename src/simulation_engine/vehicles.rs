use crate::simulation_engine::intersection::IntersectionId;

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
    /// Unique identifier for the vehicle.
    pub id: u64,
    /// The type of vehicle (Car, Bus, Truck, etc.).
    pub vehicle_type: VehicleType,
    /// The designated entry intersection where the vehicle starts.
    pub entry_point: IntersectionId,
    /// The designated exit intersection where the vehicle intends to leave.
    pub exit_point: IntersectionId,
    /// Current speed of the vehicle (units per second).
    pub speed: f64,
    /// Physical length of the vehicle in meters.
    pub length: f64,
    /// Priority (e.g., 0 for normal, higher for emergency vehicles).
    pub priority: u8,
    /// Whether this vehicle is an emergency vehicle.
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
        priority: u8,
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
            priority,
            is_emergency,
        }
    }
}

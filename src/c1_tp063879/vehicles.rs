use crate::c1_tp063879::intersections::IntersectionId;

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
    pub is_in_lane: bool,
    pub is_accident: bool,
    pub severity: i8,
    pub accident_timestamp: Option<u64>,
    pub waiting_time: u64,
    pub waiting_start: Option<u64>,
}

impl Vehicle {
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
            is_in_lane: false,
            is_accident: false,
            severity: 0,
            accident_timestamp: None,
            waiting_time: 0,
            waiting_start: None,
        }
    }

    // Returns true if the vehicle is an emergency vehicle.
    pub fn is_emergency(&self) -> bool {
        self.vehicle_type == VehicleType::EmergencyVan
    }
}

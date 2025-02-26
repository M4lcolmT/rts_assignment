pub mod vehicles {
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
    }

    impl Vehicle {
        pub fn new(id: u64, vehicle_type: VehicleType, entry_point: IntersectionId, exit_point: IntersectionId, speed: f64) -> Self {
            let (length, is_emergency) = match vehicle_type {
                VehicleType::Car => (4.5, false),
                VehicleType::Bus => (12.0, false),
                VehicleType::Truck => (16.0, false),
                VehicleType::EmergencyVan => (5.5, true),
            };
            Self { id, vehicle_type, entry_point, exit_point, speed, length, is_emergency }
        }
    }
}

// ---------- simulation_engine::route_generation ----------
pub mod route_generation {
    use crate::simulation_engine::intersections::IntersectionId;
    use crate::simulation_engine::lanes::Lane;
    // A dummy route generation function.
    pub fn generate_shortest_lane_route(lanes: &Vec<Lane>, entry: IntersectionId, exit: IntersectionId) -> Option<Vec<Lane>> {
        // For demonstration, if a lane exists that goes from entry to exit, return it.
        for lane in lanes {
            if lane.from == entry && lane.to == exit {
                return Some(vec![lane.clone()]);
            }
        }
        // Otherwise, return the first lane as a fallback.
        lanes.first().map(|lane| vec![lane.clone()])
    }
}
use crate::simulation_engine::intersections::IntersectionId;
use crate::simulation_engine::vehicles::Vehicle;

/// A lane connecting two intersections (or an outside entry/exit).
#[derive(Debug, Clone)]
pub struct Lane {
    /// A descriptive name for debugging or logging.
    pub name: String,
    /// The intersection from which the lane starts.
    pub from: IntersectionId,
    /// The intersection to which the lane goes.
    pub to: IntersectionId,
    /// Length of the lane in meters.
    pub length_meters: f64,
    /// Current occupied vehicle length in meters (to model capacity).
    pub current_vehicle_length: f64,
    /// Whether an emergency vehicle is present on this lane.
    pub has_emergency_vehicle: bool,
    pub waiting_time: f64,
}

impl Lane {
    pub fn new(name: String, from: IntersectionId, to: IntersectionId, length_meters: f64) -> Self {
        Self {
            name,
            from,
            to,
            length_meters,
            current_vehicle_length: 0.0,
            has_emergency_vehicle: false,
            waiting_time: 0.0,
        }
    }

    /// Check if there is space for a new vehicle.
    pub fn can_add_vehicle(&self, vehicle: &Vehicle) -> bool {
        // If an emergency vehicle is present, it blocks or overrides usage.
        self.has_emergency_vehicle
            || (self.current_vehicle_length + vehicle.length <= self.length_meters)
    }

    /// Attempt to add a vehicle onto this lane.
    pub fn add_vehicle(&mut self, vehicle: &Vehicle) -> bool {
        if vehicle.is_emergency {
            self.has_emergency_vehicle = true;
            true
        } else if self.can_add_vehicle(vehicle) {
            self.current_vehicle_length += vehicle.length;
            true
        } else {
            false
        }
    }

    /// Remove a vehicle from this lane.
    pub fn remove_vehicle(&mut self, vehicle: &Vehicle) {
        if self.current_vehicle_length >= vehicle.length {
            self.current_vehicle_length -= vehicle.length;
        }
        if vehicle.is_emergency {
            self.has_emergency_vehicle = false;
        }
    }
}

pub fn create_lanes() -> Vec<Lane> {
    vec![
        Lane::new(
            "(0,0) -> (0,1)".to_string(),
            IntersectionId(0, 0),
            IntersectionId(0, 1),
            300.0,
        ),
        Lane::new(
            "(0,1) -> (0,0)".to_string(),
            IntersectionId(0, 1),
            IntersectionId(0, 0),
            300.0,
        ),
        Lane::new(
            "(0,1) -> (0,2)".to_string(),
            IntersectionId(0, 1),
            IntersectionId(0, 2),
            200.0,
        ),
        Lane::new(
            "(0,2) -> (0,1)".to_string(),
            IntersectionId(0, 2),
            IntersectionId(0, 1),
            200.0,
        ),
        Lane::new(
            "(0,2) -> (0,3)".to_string(),
            IntersectionId(0, 2),
            IntersectionId(0, 3),
            400.0,
        ),
        Lane::new(
            "(0,3) -> (0,2)".to_string(),
            IntersectionId(0, 3),
            IntersectionId(0, 2),
            400.0,
        ),
        Lane::new(
            "(1,0) -> (1,1)".to_string(),
            IntersectionId(1, 0),
            IntersectionId(1, 1),
            150.0,
        ),
        Lane::new(
            "(1,1) -> (1,0)".to_string(),
            IntersectionId(1, 1),
            IntersectionId(1, 0),
            150.0,
        ),
        Lane::new(
            "(1,1) -> (1,2)".to_string(),
            IntersectionId(1, 1),
            IntersectionId(1, 2),
            600.0,
        ),
        Lane::new(
            "(1,2) -> (1,1)".to_string(),
            IntersectionId(1, 2),
            IntersectionId(1, 1),
            600.0,
        ),
        Lane::new(
            "(1,2) -> (1,3)".to_string(),
            IntersectionId(1, 2),
            IntersectionId(1, 3),
            650.0,
        ),
        Lane::new(
            "(1,3) -> (1,2)".to_string(),
            IntersectionId(1, 3),
            IntersectionId(1, 2),
            650.0,
        ),
        Lane::new(
            "(2,0) -> (2,1)".to_string(),
            IntersectionId(2, 0),
            IntersectionId(2, 1),
            800.0,
        ),
        Lane::new(
            "(2,1) -> (2,0)".to_string(),
            IntersectionId(2, 1),
            IntersectionId(2, 0),
            800.0,
        ),
        Lane::new(
            "(2,1) -> (2,2)".to_string(),
            IntersectionId(2, 1),
            IntersectionId(2, 2),
            500.0,
        ),
        Lane::new(
            "(2,2) -> (2,1)".to_string(),
            IntersectionId(2, 2),
            IntersectionId(2, 1),
            500.0,
        ),
        Lane::new(
            "(2,2) -> (2,3)".to_string(),
            IntersectionId(2, 2),
            IntersectionId(2, 3),
            450.0,
        ),
        Lane::new(
            "(2,3) -> (2,2)".to_string(),
            IntersectionId(2, 3),
            IntersectionId(2, 2),
            450.0,
        ),
        Lane::new(
            "(3,0) -> (3,1)".to_string(),
            IntersectionId(3, 0),
            IntersectionId(3, 1),
            200.0,
        ),
        Lane::new(
            "(3,1) -> (3,0)".to_string(),
            IntersectionId(3, 1),
            IntersectionId(3, 0),
            200.0,
        ),
        Lane::new(
            "(3,1) -> (3,2)".to_string(),
            IntersectionId(3, 1),
            IntersectionId(3, 2),
            550.0,
        ),
        Lane::new(
            "(3,2) -> (3,1)".to_string(),
            IntersectionId(3, 2),
            IntersectionId(3, 1),
            550.0,
        ),
        Lane::new(
            "(3,2) -> (3,3)".to_string(),
            IntersectionId(3, 2),
            IntersectionId(3, 3),
            750.0,
        ),
        Lane::new(
            "(3,3) -> (3,2)".to_string(),
            IntersectionId(3, 3),
            IntersectionId(3, 2),
            750.0,
        ),
        // Now we add vertical connections to make the grid fully adjacent:

        // Column 0
        Lane::new(
            "(0,0) -> (1,0)".to_string(),
            IntersectionId(0, 0),
            IntersectionId(1, 0),
            300.0,
        ),
        Lane::new(
            "(1,0) -> (0,0)".to_string(),
            IntersectionId(1, 0),
            IntersectionId(0, 0),
            300.0,
        ),
        Lane::new(
            "(1,0) -> (2,0)".to_string(),
            IntersectionId(1, 0),
            IntersectionId(2, 0),
            200.0,
        ),
        Lane::new(
            "(2,0) -> (1,0)".to_string(),
            IntersectionId(2, 0),
            IntersectionId(1, 0),
            200.0,
        ),
        Lane::new(
            "(2,0) -> (3,0)".to_string(),
            IntersectionId(2, 0),
            IntersectionId(3, 0),
            250.0,
        ),
        Lane::new(
            "(3,0) -> (2,0)".to_string(),
            IntersectionId(3, 0),
            IntersectionId(2, 0),
            250.0,
        ),
        // Column 1
        Lane::new(
            "(0,1) -> (1,1)".to_string(),
            IntersectionId(0, 1),
            IntersectionId(1, 1),
            400.0,
        ),
        Lane::new(
            "(1,1) -> (0,1)".to_string(),
            IntersectionId(1, 1),
            IntersectionId(0, 1),
            400.0,
        ),
        Lane::new(
            "(1,1) -> (2,1)".to_string(),
            IntersectionId(1, 1),
            IntersectionId(2, 1),
            150.0,
        ),
        Lane::new(
            "(2,1) -> (1,1)".to_string(),
            IntersectionId(2, 1),
            IntersectionId(1, 1),
            150.0,
        ),
        Lane::new(
            "(2,1) -> (3,1)".to_string(),
            IntersectionId(2, 1),
            IntersectionId(3, 1),
            600.0,
        ),
        Lane::new(
            "(3,1) -> (2,1)".to_string(),
            IntersectionId(3, 1),
            IntersectionId(2, 1),
            600.0,
        ),
        // Column 2
        Lane::new(
            "(0,2) -> (1,2)".to_string(),
            IntersectionId(0, 2),
            IntersectionId(1, 2),
            700.0,
        ),
        Lane::new(
            "(1,2) -> (0,2)".to_string(),
            IntersectionId(1, 2),
            IntersectionId(0, 2),
            700.0,
        ),
        Lane::new(
            "(1,2) -> (2,2)".to_string(),
            IntersectionId(1, 2),
            IntersectionId(2, 2),
            550.0,
        ),
        Lane::new(
            "(2,2) -> (1,2)".to_string(),
            IntersectionId(2, 2),
            IntersectionId(1, 2),
            550.0,
        ),
        Lane::new(
            "(2,2) -> (3,2)".to_string(),
            IntersectionId(2, 2),
            IntersectionId(3, 2),
            300.0,
        ),
        Lane::new(
            "(3,2) -> (2,2)".to_string(),
            IntersectionId(3, 2),
            IntersectionId(2, 2),
            300.0,
        ),
        // Column 3
        Lane::new(
            "(0,3) -> (1,3)".to_string(),
            IntersectionId(0, 3),
            IntersectionId(1, 3),
            100.0,
        ),
        Lane::new(
            "(1,3) -> (0,3)".to_string(),
            IntersectionId(1, 3),
            IntersectionId(0, 3),
            100.0,
        ),
        Lane::new(
            "(1,3) -> (2,3)".to_string(),
            IntersectionId(1, 3),
            IntersectionId(2, 3),
            200.0,
        ),
        Lane::new(
            "(2,3) -> (1,3)".to_string(),
            IntersectionId(2, 3),
            IntersectionId(1, 3),
            200.0,
        ),
        Lane::new(
            "(2,3) -> (3,3)".to_string(),
            IntersectionId(2, 3),
            IntersectionId(3, 3),
            400.0,
        ),
        Lane::new(
            "(3,3) -> (2,3)".to_string(),
            IntersectionId(3, 3),
            IntersectionId(2, 3),
            400.0,
        ),
    ]
}

use crate::simulation_engine::intersections::IntersectionId;
use crate::simulation_engine::vehicles::Vehicle;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct Lane {
    pub name: String,
    pub from: IntersectionId,
    pub to: IntersectionId,
    pub length_meters: f64,
    /// Current occupied vehicle length in meters (to model capacity).
    pub current_vehicle_length: f64,
    pub has_emergency_vehicle: bool,
    pub has_accident: bool,
    pub waiting_time: f64,
    /// FIFO queue to store vehicles on the lane.
    pub vehicle_queue: VecDeque<Vehicle>,
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
            has_accident: false,
            waiting_time: 0.0,
            vehicle_queue: VecDeque::new(),
        }
    }

    /// Check if there is space for a new vehicle.
    /// Note: If an emergency vehicle is already present the lane is blocked.
    pub fn can_add_vehicle(&self, vehicle: &Vehicle) -> bool {
        if self.has_emergency_vehicle {
            return false;
        }
        self.current_vehicle_length + vehicle.length <= self.length_meters
    }

    /// Attempt to add a vehicle onto this lane.
    /// The vehicle is pushed to the back of the FIFO queue.
    pub fn add_vehicle(&mut self, vehicle: &Vehicle) -> bool {
        if vehicle.is_emergency() {
            self.has_emergency_vehicle = true;
            self.vehicle_queue.push_back(vehicle.clone());
            true
        } else if self.can_add_vehicle(vehicle) {
            self.current_vehicle_length += vehicle.length;
            self.vehicle_queue.push_back(vehicle.clone());
            true
        } else {
            false
        }
    }

    /// Remove a vehicle from this lane.
    /// In FIFO operation the vehicle at the front is normally removed.
    pub fn remove_vehicle(&mut self, vehicle: &Vehicle) {
        if let Some(pos) = self.vehicle_queue.iter().position(|v| v.id == vehicle.id) {
            self.vehicle_queue.remove(pos);
            if !vehicle.is_emergency() {
                if self.current_vehicle_length >= vehicle.length {
                    self.current_vehicle_length -= vehicle.length;
                }
            } else {
                self.has_emergency_vehicle = false;
            }
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

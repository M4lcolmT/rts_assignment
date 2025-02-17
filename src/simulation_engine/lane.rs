use crate::simulation_engine::intersection::IntersectionId;
use crate::simulation_engine::vehicles::Vehicle;

/// Represents the state of a lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaneType {
    /// Standard bidirectional lane.
    TwoWay,
    /// One-way entry into an intersection.
    OneWayEntry,
    /// One-way exit from an intersection.
    OneWayExit,
}

/// Represents a lane (road connection between intersections).
#[derive(Debug, Clone)]
pub struct Lane {
    /// Start intersection (node).
    pub from: IntersectionId,
    /// End intersection (node).
    pub to: IntersectionId,
    /// Type of lane (one-way or two-way).
    pub lane_type: LaneType,
    /// Length of the lane in meters.
    pub length_meters: f64,
    /// Current occupied vehicle length in meters.
    pub current_vehicle_length: f64,
    /// Flag to indicate if an emergency vehicle is present.
    pub has_emergency_vehicle: bool,
}

impl Lane {
    /// Creates a new lane between two intersections.
    pub fn new(
        from: IntersectionId,
        to: IntersectionId,
        lane_type: LaneType,
        length_meters: f64,
    ) -> Self {
        Self {
            from,
            to,
            lane_type,
            length_meters,
            current_vehicle_length: 0.0,
            has_emergency_vehicle: false,
        }
    }

    /// Checks if a new vehicle can enter the lane.
    pub fn can_add_vehicle(&self, vehicle: &Vehicle) -> bool {
        self.has_emergency_vehicle
            || self.current_vehicle_length + vehicle.length <= self.length_meters
    }

    /// Adds a vehicle to the lane if there is enough space.
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

    /// Removes a vehicle from the lane.
    pub fn remove_vehicle(&mut self, vehicle: &Vehicle) {
        if self.current_vehicle_length >= vehicle.length {
            self.current_vehicle_length -= vehicle.length;
        }
        if vehicle.is_emergency {
            self.has_emergency_vehicle = false;
        }
    }
}

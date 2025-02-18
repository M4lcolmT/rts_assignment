use crate::simulation_engine::intersections::IntersectionId;
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

// Note that two way lane of outer intersections, the "to" coordinates are always (-1,-1)

pub fn create_lanes() -> Vec<Lane> {
    vec![
        Lane::new(
            IntersectionId(-1, -1),
            IntersectionId(0, 0),
            LaneType::OneWayEntry,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 0),
            IntersectionId(0, 1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 0),
            IntersectionId(1, 0),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 1),
            IntersectionId(-1, -1),
            LaneType::OneWayExit,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 1),
            IntersectionId(0, 2),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 1),
            IntersectionId(1, 1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(-1, -1),
            IntersectionId(0, 2),
            LaneType::OneWayEntry,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 2),
            IntersectionId(0, 3),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 2),
            IntersectionId(1, 2),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 3),
            IntersectionId(-1, -1),
            LaneType::OneWayExit,
            100.0,
        ),
        Lane::new(
            IntersectionId(0, 3),
            IntersectionId(1, 3),
            LaneType::TwoWay,
            100.0,
        ),
        // two way lane for outer intersections
        Lane::new(
            IntersectionId(1, 0),
            IntersectionId(-1, -1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(1, 0),
            IntersectionId(1, 1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(1, 0),
            IntersectionId(2, 0),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(1, 1),
            IntersectionId(1, 2),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(1, 1),
            IntersectionId(2, 1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(1, 2),
            IntersectionId(1, 3),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(1, 2),
            IntersectionId(2, 2),
            LaneType::TwoWay,
            100.0,
        ),
        // two way lane for outer intersections
        Lane::new(
            IntersectionId(1, 3),
            IntersectionId(-1, -1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(1, 3),
            IntersectionId(2, 3),
            LaneType::TwoWay,
            100.0,
        ),
        // two way lane for outer intersections
        Lane::new(
            IntersectionId(2, 0),
            IntersectionId(-1, -1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(2, 0),
            IntersectionId(2, 1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(2, 0),
            IntersectionId(3, 0),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(2, 1),
            IntersectionId(2, 2),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(2, 1),
            IntersectionId(3, 1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(2, 2),
            IntersectionId(2, 3),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(2, 2),
            IntersectionId(3, 2),
            LaneType::TwoWay,
            100.0,
        ),
        // two way lane for outer intersections
        Lane::new(
            IntersectionId(2, 3),
            IntersectionId(-1, -1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(2, 3),
            IntersectionId(3, 3),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(3, 0),
            IntersectionId(-1, -1),
            LaneType::OneWayExit,
            100.0,
        ),
        Lane::new(
            IntersectionId(3, 0),
            IntersectionId(3, 1),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(3, 1),
            IntersectionId(-1, -1),
            LaneType::OneWayExit,
            100.0,
        ),
        Lane::new(
            IntersectionId(3, 1),
            IntersectionId(3, 2),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(-1, -1),
            IntersectionId(3, 2),
            LaneType::OneWayEntry,
            100.0,
        ),
        Lane::new(
            IntersectionId(3, 2),
            IntersectionId(3, 3),
            LaneType::TwoWay,
            100.0,
        ),
        Lane::new(
            IntersectionId(-1, -1),
            IntersectionId(3, 3),
            LaneType::OneWayEntry,
            100.0,
        ),
    ]
}

use crate::simulation_engine::intersection::{Intersection, IntersectionControl, IntersectionId};
use crate::simulation_engine::lane::{Lane, LaneType};
use std::collections::HashMap;

/// Represents the entire 4x4 traffic grid.
pub struct TrafficGrid {
    /// Stores all intersections by their (row, col) ID.
    /// (Only grid intersections, not virtual nodes.)
    pub intersections: HashMap<IntersectionId, Intersection>,
    /// Stores all lanes by their (from → to) relationship.
    pub lanes: Vec<Lane>,
}

impl TrafficGrid {
    /// Initializes a 4x4 traffic grid with predefined intersections and lanes,
    /// including outer lanes using virtual nodes.
    pub fn new() -> Self {
        let mut intersections = HashMap::new();
        let mut lanes = Vec::new();

        // --- Create Real Grid Intersections (rows 0..3, cols 0..3) ---
        for row in 0..4 {
            for col in 0..4 {
                // Set entry/exit properties per your design
                let is_entry = matches!((row, col), (0, 1) | (0, 2) | (3, 1) | (3, 2));
                let is_exit = matches!((row, col), (0, 2) | (3, 1) | (3, 2));

                // For demonstration, assign traffic light control at selected nodes.
                let control = if matches!((row, col), (1, 2) | (2, 2) | (3, 2)) {
                    IntersectionControl::TrafficLight
                } else {
                    IntersectionControl::Normal
                };

                // Create the intersection and add it to the grid.
                let intersection =
                    Intersection::new(row as i8, col as i8, is_entry, is_exit, control, Vec::new());
                intersections.insert(intersection.id, intersection);
            }
        }

        let lane_length = 30.0; // Standard lane length in meters

        // --- Internal Lanes: Horizontal & Vertical (TwoWay) ---

        // Horizontal lanes: for each row, connect (row, col) to (row, col+1)
        for row in 0..4 {
            for col in 0..3 {
                let from = IntersectionId(row as i8, col as i8);
                let to = IntersectionId(row as i8, (col + 1) as i8);
                lanes.push(Lane::new(from, to, LaneType::TwoWay, lane_length));
                lanes.push(Lane::new(to, from, LaneType::TwoWay, lane_length));
            }
        }

        // Vertical lanes: for each column, connect (row, col) to (row+1, col)
        for col in 0..4 {
            for row in 0..3 {
                let from = IntersectionId(row as i8, col as i8);
                let to = IntersectionId((row + 1) as i8, col as i8);
                lanes.push(Lane::new(from, to, LaneType::TwoWay, lane_length));
                lanes.push(Lane::new(to, from, LaneType::TwoWay, lane_length));
            }
        }

        // --- Outer Lanes: Using Virtual Nodes ---
        // Virtual nodes have coordinates outside [0,3]. For instance:
        // Top: row = -1, Bottom: row = 4, Left: col = -1, Right: col = 4.

        // Top outer lanes: Connect virtual node (-1, col) <-> grid (0, col)
        for col in 0..4 {
            let virtual_node = IntersectionId(-1, col as i8);
            let grid_node = IntersectionId(0, col as i8);
            // Lane entering the grid: from virtual to grid (OneWayEntry)
            lanes.push(Lane::new(
                virtual_node,
                grid_node,
                LaneType::OneWayEntry,
                lane_length,
            ));
            // Lane exiting the grid: from grid to virtual (OneWayExit)
            lanes.push(Lane::new(
                grid_node,
                virtual_node,
                LaneType::OneWayExit,
                lane_length,
            ));
        }

        // Bottom outer lanes: Connect virtual node (4, col) <-> grid (3, col)
        for col in 0..4 {
            let virtual_node = IntersectionId(4, col as i8);
            let grid_node = IntersectionId(3, col as i8);
            lanes.push(Lane::new(
                virtual_node,
                grid_node,
                LaneType::OneWayEntry,
                lane_length,
            ));
            lanes.push(Lane::new(
                grid_node,
                virtual_node,
                LaneType::OneWayExit,
                lane_length,
            ));
        }

        // Left outer lanes: Connect virtual node (row, -1) <-> grid (row, 0)
        for row in 0..4 {
            let virtual_node = IntersectionId(row as i8, -1);
            let grid_node = IntersectionId(row as i8, 0);
            lanes.push(Lane::new(
                virtual_node,
                grid_node,
                LaneType::OneWayEntry,
                lane_length,
            ));
            lanes.push(Lane::new(
                grid_node,
                virtual_node,
                LaneType::OneWayExit,
                lane_length,
            ));
        }

        // Right outer lanes: Connect virtual node (row, 4) <-> grid (row, 3)
        for row in 0..4 {
            let virtual_node = IntersectionId(row as i8, 4);
            let grid_node = IntersectionId(row as i8, 3);
            lanes.push(Lane::new(
                virtual_node,
                grid_node,
                LaneType::OneWayEntry,
                lane_length,
            ));
            lanes.push(Lane::new(
                grid_node,
                virtual_node,
                LaneType::OneWayExit,
                lane_length,
            ));
        }

        // --- Update Intersections' Connections ---
        // For each lane, if the "from" node is an actual grid intersection, add the "to" node to its connectivity list.
        for lane in &lanes {
            let IntersectionId(row, col) = lane.from;
            if row >= 0 && row <= 3 && col >= 0 && col <= 3 {
                if let Some(intersection) = intersections.get_mut(&lane.from) {
                    intersection.connected.push(lane.to);
                }
            }
        }

        TrafficGrid {
            intersections,
            lanes,
        }
    }

    /// Retrieves an intersection by its ID.
    pub fn get_intersection(&self, id: &IntersectionId) -> Option<&Intersection> {
        self.intersections.get(id)
    }

    /// Retrieves all lanes that originate from a given intersection.
    pub fn get_lanes_from(&self, id: &IntersectionId) -> Vec<&Lane> {
        self.lanes.iter().filter(|lane| &lane.from == id).collect()
    }
}

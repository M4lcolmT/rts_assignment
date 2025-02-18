use crate::simulation_engine::intersections::{create_intersections, Intersection, IntersectionId};
use crate::simulation_engine::lanes::{create_lanes, Lane};
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

        for i in create_intersections() {
            intersections.insert(i.id, i);
        }

        for i in create_lanes() {
            lanes.push(i);
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

// src/intersections.rs
use crate::data_structures::{
    Intersection, IntersectionDirection, IntersectionId, IntersectionRole, LightState,
};

pub fn all_intersections() -> Vec<Intersection> {
    let intersection1 = Intersection::new(
        0,
        0,
        IntersectionRole::Normal,
        IntersectionDirection::OneWayOut,
        vec![IntersectionId(0, 1), IntersectionId(1, 0)],
    );
}

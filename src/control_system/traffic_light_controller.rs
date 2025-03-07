use crate::simulation_engine::intersections::{Intersection, IntersectionControl, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TrafficLightPhase {
    pub green_lanes: Vec<String>,
    pub duration: u64,
}

pub struct IntersectionController {
    pub intersection: Intersection,
    pub phases: Vec<TrafficLightPhase>,
    pub current_phase_index: usize,
    pub elapsed_in_phase: u64,
    pub all_lanes: Vec<String>,
    pub emergency_override: Option<Vec<String>>,
}

impl IntersectionController {
    pub fn new(
        intersection: Intersection,
        phases: Vec<TrafficLightPhase>,
        all_lanes: Vec<String>,
    ) -> Self {
        Self {
            intersection,
            phases,
            current_phase_index: 0,
            elapsed_in_phase: 0,
            all_lanes,
            emergency_override: None,
        }
    }

    /// Update the phase timer if no emergency override is active.
    pub fn update(&mut self) {
        if self.emergency_override.is_some() {
            // Skip normal phase cycling during emergency override
            return;
        }
        self.elapsed_in_phase += 1;
        let current_phase = &self.phases[self.current_phase_index];
        if self.elapsed_in_phase >= current_phase.duration {
            self.elapsed_in_phase = 0;
            self.current_phase_index = (self.current_phase_index + 1) % self.phases.len();
            self.apply_current_phase();
        }
    }

    /// Apply current phase or emergency override
    pub fn apply_current_phase(&self) {
        if let Some(ref override_lanes) = self.emergency_override {
            // If there's an emergency override, *only* these lanes are green
            let red_lanes: Vec<String> = self
                .all_lanes
                .iter()
                .filter(|lane| !override_lanes.contains(lane))
                .cloned()
                .collect();

            println!(
                "Intersection {:?} emergency override: Green for lanes: {:?} and Red for lanes: {:?}",
                self.intersection.id, override_lanes, red_lanes
            );
        } else {
            // Normal phase
            let current_green = &self.phases[self.current_phase_index].green_lanes;
            let red_lanes: Vec<String> = self
                .all_lanes
                .iter()
                .filter(|lane| !current_green.contains(lane))
                .cloned()
                .collect();

            println!(
                "Intersection {:?} switching to phase {}: Green for lanes: {:?} and Red for lanes: {:?}",
                self.intersection.id,
                self.current_phase_index,
                current_green,
                red_lanes
            );
        }
    }

    pub fn set_emergency_override(&mut self, lanes_to_green: Vec<String>) {
        self.emergency_override = Some(lanes_to_green);
        self.apply_current_phase();
    }

    pub fn clear_emergency_override(&mut self) {
        if self.emergency_override.is_some() {
            println!(
                "Clearing emergency override for intersection {:?}",
                self.intersection.id
            );
        }
        self.emergency_override = None;
        self.apply_current_phase();
    }
}

pub struct TrafficLightController {
    pub controllers: HashMap<IntersectionId, IntersectionController>,
}

impl TrafficLightController {
    pub fn initialize(intersections: Vec<Intersection>, lanes: &[Lane]) -> Self {
        let mut controllers = HashMap::new();

        for intersection in intersections {
            if intersection.control == IntersectionControl::TrafficLight {
                let connected_lanes: Vec<String> = lanes
                    .iter()
                    .filter(|lane| lane.from == intersection.id)
                    .map(|lane| lane.name.clone())
                    .collect();

                if connected_lanes.is_empty() {
                    continue;
                }

                // One phase per lane for demonstration
                let phases: Vec<TrafficLightPhase> = connected_lanes
                    .iter()
                    .map(|ln| TrafficLightPhase {
                        green_lanes: vec![ln.clone()],
                        duration: 5,
                    })
                    .collect();

                let controller = IntersectionController::new(
                    intersection.clone(),
                    phases,
                    connected_lanes.clone(),
                );
                controllers.insert(intersection.id, controller);
            }
        }

        Self { controllers }
    }

    /// Update all intersections if no emergency override
    pub fn update_all(&mut self) {
        for controller in self.controllers.values_mut() {
            controller.update();
        }
    }

    /// Check if lane is green for the intersection
    pub fn is_lane_green(&self, intersection_id: IntersectionId, lane_name: &str) -> bool {
        if let Some(ctrl) = self.controllers.get(&intersection_id) {
            // If there's an override, only lanes in that override are green
            if let Some(ref override_lanes) = ctrl.emergency_override {
                return override_lanes.contains(&lane_name.to_string());
            }
            // Otherwise, check the normal phase
            let current_phase = &ctrl.phases[ctrl.current_phase_index];
            current_phase.green_lanes.contains(&lane_name.to_string())
        } else {
            true
        }
    }

    /// Determine which lanes are non-conflicting with the given "emergency lane" and set them green.
    /// For a simple “two‐way” logic, we let the lane from->to and the lane to->from be green.
    pub fn set_emergency_override(
        &mut self,
        intersection_id: IntersectionId,
        emergency_lane: &str,
        all_lanes: &[Lane],
    ) {
        if let Some(ctrl) = self.controllers.get_mut(&intersection_id) {
            let mut green_lanes = vec![];

            // 1) Always include the emergency lane
            green_lanes.push(emergency_lane.to_string());

            // 2) Also allow the opposite direction (if it exists) to be green
            //    if it doesn’t conflict. That means if the emergency lane is
            //    "(2,0) -> (3,0)", we also allow "(3,0) -> (2,0)" if that lane
            //    belongs to this same intersection.
            if let Some(em_lane_obj) = all_lanes.iter().find(|l| l.name == emergency_lane) {
                // Opposite lane name might be something like "(3,0) -> (2,0)"
                let opposite_name = format!(
                    "({},{}) -> ({},{})",
                    em_lane_obj.to.0, em_lane_obj.to.1, em_lane_obj.from.0, em_lane_obj.from.1
                );
                // Check if this intersection actually controls that lane
                if ctrl.all_lanes.contains(&opposite_name) {
                    green_lanes.push(opposite_name);
                }
            }

            ctrl.set_emergency_override(green_lanes);
        }
    }

    /// Clear override
    pub fn clear_emergency_override(&mut self, intersection_id: IntersectionId) {
        if let Some(ctrl) = self.controllers.get_mut(&intersection_id) {
            ctrl.clear_emergency_override();
        }
    }
}

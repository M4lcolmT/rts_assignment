// traffic_light_controller.rs
use crate::simulation_engine::intersections::{Intersection, IntersectionControl, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use std::collections::HashMap;

/// Represents a single traffic light phase at an intersection.
/// For example, in this phase the specified lane(s) get the green light.
#[derive(Debug, Clone)]
pub struct TrafficLightPhase {
    /// The lane identifiers (e.g., lane names) that are green during this phase.
    pub green_lanes: Vec<String>,
    /// Duration of this phase in simulation ticks.
    pub duration: u64,
}

/// Controller for an individual intersection.
/// It manages the phases and current state of the intersection's traffic lights.
pub struct IntersectionController {
    pub intersection: Intersection,
    pub phases: Vec<TrafficLightPhase>,
    pub current_phase_index: usize,
    pub elapsed_in_phase: u64,
    /// A list of all lane names connected to this intersection.
    pub all_lanes: Vec<String>,
}

impl IntersectionController {
    /// Create a new controller for an intersection with a set of phases and the full set of connected lanes.
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
        }
    }

    /// Update the phase timer; if the current phase has run its course, move to the next phase.
    pub fn update(&mut self) {
        self.elapsed_in_phase += 1;
        let current_phase = &self.phases[self.current_phase_index];
        if self.elapsed_in_phase >= current_phase.duration {
            self.elapsed_in_phase = 0;
            self.current_phase_index = (self.current_phase_index + 1) % self.phases.len();
            self.apply_current_phase();
        }
    }

    /// Apply the current phase and print both green and red lanes.
    pub fn apply_current_phase(&mut self) {
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

/// Global traffic light controller that manages all intersections with traffic lights.
pub struct TrafficLightController {
    pub controllers: HashMap<IntersectionId, IntersectionController>,
}

impl TrafficLightController {
    /// Initialize the traffic light controllers for each intersection that uses traffic light control.
    /// This function uses the lanes to determine which lanes are connected to each intersection.
    pub fn initialize(intersections: Vec<Intersection>, lanes: &[Lane]) -> Self {
        let mut controllers = HashMap::new();
        for intersection in intersections {
            if intersection.control == IntersectionControl::TrafficLight {
                // Identify lanes that originate from this intersection.
                let connected_lanes: Vec<String> = lanes
                    .iter()
                    .filter(|lane| lane.from == intersection.id)
                    .map(|lane| lane.name.clone())
                    .collect();

                // Skip the intersection if no connected lanes are found.
                if connected_lanes.is_empty() {
                    continue;
                }

                // For simplicity, create one phase per connected lane.
                // Each phase grants a green light to one lane.
                let phases: Vec<TrafficLightPhase> = connected_lanes
                    .iter()
                    .map(|lane_name| TrafficLightPhase {
                        green_lanes: vec![lane_name.clone()],
                        duration: 5, // e.g., 5 ticks per phase
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

    /// Update all managed intersection controllers; typically called once per simulation tick.
    pub fn update_all(&mut self) {
        for controller in self.controllers.values_mut() {
            controller.update();
        }
    }

    // TODO: not sure if this can be improved or not. Introduce lane ID instead of name matching?
    /// Returns true if the lane (identified by its name) is currently green
    /// for the intersection with the provided id.
    pub fn is_lane_green(&self, intersection_id: IntersectionId, lane_name: &str) -> bool {
        if let Some(controller) = self.controllers.get(&intersection_id) {
            let current_phase = &controller.phases[controller.current_phase_index];
            current_phase.green_lanes.contains(&lane_name.to_string())
        } else {
            true // If no controller is found, assume no control.
        }
    }
}

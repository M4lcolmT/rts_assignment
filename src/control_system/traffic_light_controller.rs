mod control_system {
    pub mod traffic_light_controller {
        use std::{collections::HashMap, sync::{Arc, Mutex}, thread, time::Duration};
        use crate::simulation_engine::intersections::{Intersection, IntersectionControl, IntersectionId, LightState};

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum FourPhase {
            Phase1,
            Phase2,
            Phase3,
            Phase4,
        }

        #[derive(Debug, Clone)]
        pub struct IntersectionPhaseState {
            pub current_phase: FourPhase,
            pub time_remaining: u64,
            pub phase_durations: HashMap<FourPhase, u64>,
            pub emergency_override: bool,
            pub emergency_ns_green: bool,
            pub emergency_ew_green: bool,
        }

        impl IntersectionPhaseState {
            pub fn new() -> Self {
                let mut phase_durations = HashMap::new();
                phase_durations.insert(FourPhase::Phase1, 10);
                phase_durations.insert(FourPhase::Phase2, 5);
                phase_durations.insert(FourPhase::Phase3, 10);
                phase_durations.insert(FourPhase::Phase4, 5);
                Self {
                    current_phase: FourPhase::Phase1,
                    time_remaining: 10,
                    phase_durations,
                    emergency_override: false,
                    emergency_ns_green: false,
                    emergency_ew_green: false,
                }
            }

            pub fn next_phase(&mut self) {
                self.current_phase = match self.current_phase {
                    FourPhase::Phase1 => FourPhase::Phase2,
                    FourPhase::Phase2 => FourPhase::Phase3,
                    FourPhase::Phase3 => FourPhase::Phase4,
                    FourPhase::Phase4 => FourPhase::Phase1,
                };
                self.time_remaining = *self.phase_durations.get(&self.current_phase).unwrap_or(&10);
            }

            fn normal_phase_lights(&self) -> (LightState, LightState) {
                match self.current_phase {
                    FourPhase::Phase1 => (LightState::Green, LightState::Red),
                    FourPhase::Phase2 => (LightState::Green, LightState::Red),
                    FourPhase::Phase3 => (LightState::Red, LightState::Green),
                    FourPhase::Phase4 => (LightState::Red, LightState::Green),
                }
            }

            fn emergency_phase_lights(&self) -> (LightState, LightState) {
                match (self.emergency_ns_green, self.emergency_ew_green) {
                    (true, false) => (LightState::Green, LightState::Red),
                    (false, true) => (LightState::Red, LightState::Green),
                    _ => (LightState::Red, LightState::Red),
                }
            }

            pub fn current_lights(&self) -> (LightState, LightState) {
                if self.emergency_override { self.emergency_phase_lights() } else { self.normal_phase_lights() }
            }
        }

        pub struct TrafficLightController {
            intersection_states: Arc<Mutex<HashMap<IntersectionId, IntersectionPhaseState>>>,
            intersections: Arc<Mutex<HashMap<IntersectionId, Intersection>>>,
        }

        impl TrafficLightController {
            pub fn new(all_intersections: Vec<Intersection>) -> Self {
                let mut phase_map = HashMap::new();
                let mut intersection_map = HashMap::new();
                for inter in all_intersections {
                    if inter.control == IntersectionControl::TrafficLight {
                        phase_map.insert(inter.id, IntersectionPhaseState::new());
                        intersection_map.insert(inter.id, inter);
                    }
                }
                TrafficLightController {
                    intersection_states: Arc::new(Mutex::new(phase_map)),
                    intersections: Arc::new(Mutex::new(intersection_map)),
                }
            }

            pub fn run(&self) {
                let states = Arc::clone(&self.intersection_states);
                let intersection_map = Arc::clone(&self.intersections);
                thread::spawn(move || loop {
                    {
                        let mut locked_states = states.lock().unwrap();
                        let mut locked_inters = intersection_map.lock().unwrap();
                        for (int_id, state) in locked_states.iter_mut() {
                            if !state.emergency_override {
                                if state.time_remaining > 0 {
                                    state.time_remaining -= 1;
                                } else {
                                    state.next_phase();
                                }
                            }
                            if let Some(intersection) = locked_inters.get_mut(int_id) {
                                let (ns_light, _ew_light) = state.current_lights();
                                intersection.light_state = Some(ns_light);
                            }
                        }
                    }
                    thread::sleep(Duration::from_secs(1));
                });
            }

            pub fn adjust_phase_durations(&self, intersection_id: IntersectionId, p1: u64, p2: u64, p3: u64, p4: u64) {
                let mut locked_states = self.intersection_states.lock().unwrap();
                if let Some(state) = locked_states.get_mut(&intersection_id) {
                    state.phase_durations.insert(FourPhase::Phase1, p1);
                    state.phase_durations.insert(FourPhase::Phase2, p2);
                    state.phase_durations.insert(FourPhase::Phase3, p3);
                    state.phase_durations.insert(FourPhase::Phase4, p4);
                    println!("Adjusted phase durations for {:?}: p1={}, p2={}, p3={}, p4={}", intersection_id, p1, p2, p3, p4);
                } else {
                    println!("Intersection {:?} not found or not a traffic light.", intersection_id);
                }
            }

            pub fn set_emergency_override(&self, intersection_id: IntersectionId, ns_green: bool, ew_green: bool) {
                let mut locked_states = self.intersection_states.lock().unwrap();
                if let Some(state) = locked_states.get_mut(&intersection_id) {
                    state.emergency_override = true;
                    state.emergency_ns_green = ns_green;
                    state.emergency_ew_green = ew_green;
                    println!("Intersection {:?} emergency override! N-S green={}, E-W green={}", intersection_id, ns_green, ew_green);
                }
            }

            pub fn clear_emergency_override(&self, intersection_id: IntersectionId) {
                let mut locked_states = self.intersection_states.lock().unwrap();
                if let Some(state) = locked_states.get_mut(&intersection_id) {
                    state.emergency_override = false;
                    println!("Intersection {:?} emergency override cleared!", intersection_id);
                }
            }

            pub fn check_conflicts(&self) {
                let locked_states = self.intersection_states.lock().unwrap();
                let mut phase_map: HashMap<FourPhase, Vec<IntersectionId>> = HashMap::new();
                for (id, state) in locked_states.iter() {
                    phase_map.entry(state.current_phase).or_default().push(*id);
                }
                for (phase, ids) in phase_map {
                    if ids.len() > 1 {
                        println!("Phase {:?} is active at multiple intersections: {:?}", phase, ids);
                    }
                }
            }

            pub fn ensure_fairness(&self) {
                println!("Fairness check triggered (logic not implemented).");
            }
        }
    }
}
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration};

use crate::simulation_engine::intersections::{Intersection, IntersectionControl, IntersectionId, LightState};

pub struct TrafficLightController {
    // Use IntersectionId as the key
    traffic_lights: Arc<Mutex<HashMap<IntersectionId, Intersection>>>,
}

impl TrafficLightController {
    pub fn new(intersections: Vec<Intersection>) -> Self {
        let mut light_map = HashMap::new();
        for intersection in intersections {
            if intersection.control == IntersectionControl::TrafficLight {
                light_map.insert(intersection.id, intersection.clone()); //Store a copy so the controller does not mutate original
            }
        }

        TrafficLightController {
            traffic_lights: Arc::new(Mutex::new(light_map)),
        }
    }

    pub fn run(&self) {
        let lights = Arc::clone(&self.traffic_lights);

        thread::spawn(move || {
            loop {
                let mut lights = lights.lock().unwrap();
                for (_, intersection) in lights.iter_mut() {
                    // Check if it's a traffic light intersection before updating.
                    if intersection.control == IntersectionControl::TrafficLight {
                        intersection.update_light(); // Use the intersection's update_light method
                    }
                }
                drop(lights);

                thread::sleep(Duration::from_secs(1));
            }
        });
    }

    // Adjusts traffic light timings dynamically based on recommendations from the Traffic Flow Analyzer.
    pub fn adjust_light_timing(&self, intersection_id: IntersectionId, new_state: LightState) {
        let mut lights = self.traffic_lights.lock().unwrap();
        if let Some(intersection) = lights.get_mut(&intersection_id) {
            if intersection.control == IntersectionControl::TrafficLight {
                intersection.light_state = Some(new_state);
                println!("Traffic Light {:?}: Adjusted to {:?}", intersection_id, new_state);
            } else {
                println!("Intersection {:?} is not a traffic light", intersection_id);
            }
        } else {
            println!("Intersection {:?} not found", intersection_id);
        }
    }

    // This function checks for conflicts
    pub fn check_conflicts(&self) {
        let lights = self.traffic_lights.lock().unwrap();
        let mut green_lights: Vec<IntersectionId> = Vec::new();

        // Find all green lights
        for (id, intersection) in lights.iter() {
            if intersection.control == IntersectionControl::TrafficLight && intersection.light_state == Some(LightState::Green) {
                green_lights.push(*id);
            }
        }

        // Check for conflicts (very basic example: assume any two green lights are a conflict)
        if green_lights.len() > 1 {
            println!("Potential conflict detected!");
            for id in green_lights {
                println!("Intersection {:?} is Green", id);
            }
        }
    }

    // Placeholder for fairness logic (needs more context about road priorities etc.)
    pub fn ensure_fairness(&self) {
        // This would involve more complex logic, potentially using the lanes data to understand traffic flow.
        println!("Fairness check triggered (logic needs implementation)");
    }
}

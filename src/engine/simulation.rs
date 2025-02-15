// src/engine/simulation.rs
use crate::communication::messages::SimulationMessage;
use crate::models::intersection::Intersection;
use crate::models::vehicle::{Vehicle, VehicleType};
use chrono::Local;
use rand::Rng;
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct SimulationEngine {
    pub intersections: HashMap<u32, Intersection>,
    pub vehicles: HashMap<Uuid, Vehicle>,
    pub vehicle_counters: HashMap<String, u32>,
    pub message_sender: mpsc::Sender<SimulationMessage>,
    pub simulation_time: f64,
}

impl SimulationEngine {
    pub fn new(message_sender: mpsc::Sender<SimulationMessage>) -> Self {
        let mut intersections = HashMap::new();

        // Initialize the 9 intersections with their connections based on the grid
        let intersection_connections = vec![
            (1, vec![2, 4]),
            (2, vec![1, 3, 5]),
            (3, vec![2, 6]),
            (4, vec![1, 5, 7]),
            (5, vec![2, 4, 6, 8]),
            (6, vec![3, 5, 9]),
            (7, vec![4, 8]),
            (8, vec![5, 7, 9]),
            (9, vec![6, 8]),
        ];

        for (id, connections) in intersection_connections {
            intersections.insert(
                id,
                Intersection {
                    id,
                    connected_intersections: connections,
                    current_vehicles: Vec::new(),
                    is_traffic_light_green: true,
                    max_capacity: 10,
                },
            );
        }

        SimulationEngine {
            intersections,
            vehicles: HashMap::new(),
            vehicle_counters: HashMap::new(),
            message_sender,
            simulation_time: 0.0,
        }
    }

    pub async fn spawn_vehicle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rng = rand::rng();

        // Generate vehicle type
        let vehicle_type = match rng.random_range(0..=3) {
            0 => {
                let num = self.get_next_vehicle_number("car");
                VehicleType::Car(num)
            }
            1 => {
                let num = self.get_next_vehicle_number("bus");
                VehicleType::Bus(num)
            }
            2 => {
                let num = self.get_next_vehicle_number("emergency");
                VehicleType::Emergency(num)
            }
            _ => {
                let num = self.get_next_vehicle_number("truck");
                VehicleType::Truck(num)
            }
        };

        // Generate random route
        let start_intersection = rng.random_range(1..=9);
        let mut route = vec![start_intersection];
        let mut current = start_intersection;

        // Add 2-4 more intersections to the route
        for _ in 0..rng.random_range(2..=4) {
            if let Some(intersection) = self.intersections.get(&current) {
                let next_options = &intersection.connected_intersections;
                if !next_options.is_empty() {
                    current = next_options[rng.random_range(0..next_options.len())];
                    route.push(current);
                }
            }
        }

        let vehicle = Vehicle {
            id: Uuid::new_v4(), // Requires the "v4" feature in Cargo.toml
            vehicle_type: vehicle_type.clone(),
            current_intersection: None,
            route,
            entry_time: Local::now(),
            current_speed: 1.0,
            waiting_time: 0.0,
        };

        self.message_sender
            .send(SimulationMessage::VehicleSpawned(vehicle.clone()))
            .await?;

        self.vehicles.insert(vehicle.id, vehicle);
        Ok(())
    }

    fn get_next_vehicle_number(&mut self, vehicle_type: &str) -> u32 {
        let counter = self
            .vehicle_counters
            .entry(vehicle_type.to_string())
            .or_insert(0);
        *counter += 1;
        *counter
    }

    pub async fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.simulation_time += 0.1;

        // Collect the vehicle IDs so we can iterate without holding a borrow on self.vehicles.
        let vehicle_ids: Vec<Uuid> = self.vehicles.keys().cloned().collect();

        for id in vehicle_ids {
            // Call a helper that looks up the vehicle mutably inside.
            self.process_vehicle_movement(id).await?;
        }

        Ok(())
    }

    // Change the signature to take a vehicle ID instead of a mutable reference.
    async fn process_vehicle_movement(
        &mut self,
        vehicle_id: Uuid,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Step 1: Look up the vehicle mutably in a short scope
        let (route, current_intersection) = {
            if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                (vehicle.route.clone(), vehicle.current_intersection)
            } else {
                return Ok(()); // Exit if the vehicle doesn't exist
            }
        };

        // Step 2: If the route is empty, remove the vehicle and exit
        if route.is_empty() {
            self.vehicles.remove(&vehicle_id);
            return Ok(());
        }

        // Step 3: Get the next intersection from the route
        let next_intersection = route.first().copied().unwrap();

        match current_intersection {
            None => {
                // Vehicle is entering the first intersection
                if let Some(intersection) = self.intersections.get_mut(&next_intersection) {
                    intersection.current_vehicles.push(vehicle_id);
                }
                if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                    vehicle.current_intersection = Some(next_intersection);
                }
                self.message_sender
                    .send(SimulationMessage::VehicleMoved {
                        vehicle_id,
                        from: 0,
                        to: next_intersection,
                    })
                    .await?;
            }
            Some(current) => {
                if current == next_intersection {
                    // Step 4: Advance the vehicle's route
                    let can_advance = {
                        if let Some(next) = route.get(1) {
                            if let Some(next_intersection) = self.intersections.get_mut(next) {
                                // Check if there's capacity at the next intersection
                                next_intersection.current_vehicles.len()
                                    < next_intersection.max_capacity as usize
                            } else {
                                false
                            }
                        } else {
                            true // No next intersection; vehicle completes its route
                        }
                    };

                    if can_advance {
                        // Remove vehicle from the current intersection
                        if let Some(current_intersection) = self.intersections.get_mut(&current) {
                            current_intersection
                                .current_vehicles
                                .retain(|&id| id != vehicle_id);
                        }

                        // Update vehicle's route or remove it if completed
                        if let Some(vehicle) = self.vehicles.get_mut(&vehicle_id) {
                            vehicle.route.remove(0);
                            if let Some(next) = vehicle.route.first() {
                                if let Some(next_intersection) = self.intersections.get_mut(next) {
                                    next_intersection.current_vehicles.push(vehicle_id);
                                    vehicle.current_intersection = Some(*next);
                                }
                            } else {
                                // Route completed; remove the vehicle
                                self.vehicles.remove(&vehicle_id);
                            }
                        }

                        // Send the movement message
                        let to = route.get(1).copied().unwrap_or(0);
                        self.message_sender
                            .send(SimulationMessage::VehicleMoved {
                                vehicle_id,
                                from: current,
                                to,
                            })
                            .await?;
                    }
                }
            }
        }

        Ok(())
    }
}

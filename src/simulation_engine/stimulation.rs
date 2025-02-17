// simulation.rs
//
// This module serves as the simulation engine, integrating the grid,
// route generation, vehicle movement, and intersection updates. It
// spawns vehicles with dynamically generated routes and processes
// simulation ticks to advance vehicles along their paths.

use std::thread;
use std::time::Duration;

use crate::simulation_engine::grid::TrafficGrid;
use crate::simulation_engine::intersection::IntersectionId;
use crate::simulation_engine::movement::advance_vehicle;
use crate::simulation_engine::route_generation::{generate_random_route, replan_route};
use crate::simulation_engine::vehicles::{Vehicle, VehicleType};

/// The simulation engine that holds the grid and active vehicles.
pub struct SimulationEngine {
    /// The traffic grid containing intersections and lanes.
    grid: TrafficGrid,
    /// A vector of vehicles paired with their planned routes.
    vehicles: Vec<(Vehicle, Vec<IntersectionId>)>,
}

impl SimulationEngine {
    /// Creates a new simulation engine.
    fn new() -> Self {
        // Initialize the 4x4 grid.
        let grid = TrafficGrid::new();
        let mut vehicles = Vec::new();

        // Generate a random route for a normal vehicle.
        let route1 = generate_random_route(&grid);
        if !route1.is_empty() {
            let vehicle1 = Vehicle::new(
                1,
                VehicleType::Car,
                route1.first().unwrap().clone(),
                route1.last().unwrap().clone(),
                10.0,
                0,
            );
            vehicles.push((vehicle1, route1));
        }

        // Generate a random route for an emergency vehicle.
        let route2 = generate_random_route(&grid);
        if !route2.is_empty() {
            let emergency_vehicle = Vehicle::new(
                2,
                VehicleType::EmergencyVan,
                route2.first().unwrap().clone(),
                route2.last().unwrap().clone(),
                15.0,
                10,
            );
            vehicles.push((emergency_vehicle, route2));
        }

        Self { grid, vehicles }
    }

    /// Runs the simulation loop for a given number of ticks.
    fn run(&mut self, ticks: u32) {
        // Clone lanes from the grid (used to update lane occupancy).
        let mut lanes = self.grid.lanes.clone();

        for tick in 0..ticks {
            println!("----- Tick: {} -----", tick);

            // Update all intersections (e.g., traffic lights).
            for intersection in self.grid.intersections.values_mut() {
                intersection.update_light();
            }

            // Process each vehicle's movement.
            for (vehicle, route) in self.vehicles.iter_mut() {
                if route.len() > 1 {
                    // Try to advance the vehicle from its current intersection to the next.
                    let advanced = advance_vehicle(vehicle, route, &self.grid, &mut lanes);
                    if advanced {
                        println!(
                            "Vehicle {} advanced to intersection {:?}",
                            vehicle.id, route[0]
                        );
                    } else {
                        println!(
                            "Vehicle {} waiting at intersection {:?}",
                            vehicle.id, route[0]
                        );
                        // If a vehicle is waiting for too long, you might trigger a replan.
                        // For demonstration, let's say after waiting, we replan the route.
                        // (In a full implementation, you'd use a timer or counter.)
                        let new_route = replan_route(&self.grid, route[0]);
                        println!(
                            "Vehicle {} replanned its route: {:?}",
                            vehicle.id, new_route
                        );
                        *route = new_route;
                    }
                } else {
                    // Vehicle has reached its destination.
                    println!(
                        "Vehicle {} has reached its destination at {:?}",
                        vehicle.id, route[0]
                    );
                }
            }

            // Simulate time passing between ticks (e.g., 1 second per tick).
            thread::sleep(Duration::from_secs(1));
        }

        println!("Simulation ended.");
    }
}

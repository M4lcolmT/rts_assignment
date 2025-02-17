// simulation_engine/main.rs
//
// Previously, this might have contained a standalone `fn main()`.
// We'll rename it to `run_simulation()` and make it public so that
// the top-level can call it.

use std::thread;
use std::time::Duration;

// Bring in modules from the same folder via `super` or crate path.
use crate::simulation_engine::grid::TrafficGrid;
use crate::simulation_engine::movement::advance_vehicle;
use crate::simulation_engine::route_generation::{generate_random_route, replan_route};
use crate::simulation_engine::vehicles::{Vehicle, VehicleType};

/// Run the simulation engine from here.
/// This replaces `fn main()` in the subfolder so we can call it from the top-level.
pub fn run_simulation() {
    println!("Initializing simulation engine...");

    // Create the grid
    let mut grid = TrafficGrid::new();

    // Create a lanes copy for occupancy updates
    let mut lanes = grid.lanes.clone();

    // Generate a random route for a normal vehicle
    let route1 = generate_random_route(&grid);
    let mut vehicle1 = None;
    if !route1.is_empty() {
        let v = Vehicle::new(
            1,
            VehicleType::Car,
            *route1.first().unwrap(),
            *route1.last().unwrap(),
            10.0,
            0,
        );
        vehicle1 = Some((v, route1));
    }

    // Generate a random route for an emergency vehicle
    let route2 = generate_random_route(&grid);
    let mut vehicle2 = None;
    if !route2.is_empty() {
        let ev = Vehicle::new(
            2,
            VehicleType::EmergencyVan,
            *route2.first().unwrap(),
            *route2.last().unwrap(),
            15.0,
            10,
        );
        vehicle2 = Some((ev, route2));
    }

    // Simulation loop
    let ticks = 10;
    for tick in 0..ticks {
        println!("----- Tick: {} -----", tick);

        // Update traffic lights
        for intersection in grid.intersections.values_mut() {
            intersection.update_light();
        }

        // Move vehicle1 if it exists
        if let Some((ref mut veh, ref mut route)) = vehicle1 {
            if route.len() > 1 {
                let advanced = advance_vehicle(veh, route, &grid, &mut lanes);
                if advanced {
                    println!("Vehicle {} advanced to intersection {:?}", veh.id, route[0]);
                } else {
                    println!("Vehicle {} waiting at intersection {:?}", veh.id, route[0]);
                    // Example re-route
                    let new_route = replan_route(&grid, route[0]);
                    println!("Vehicle {} replanned its route: {:?}", veh.id, new_route);
                    *route = new_route;
                }
            } else {
                println!(
                    "Vehicle {} has reached its destination at {:?}",
                    veh.id, route[0]
                );
            }
        }

        // Move vehicle2 if it exists
        if let Some((ref mut veh, ref mut route)) = vehicle2 {
            if route.len() > 1 {
                let advanced = advance_vehicle(veh, route, &grid, &mut lanes);
                if advanced {
                    println!(
                        "Emergency Vehicle {} advanced to intersection {:?}",
                        veh.id, route[0]
                    );
                } else {
                    println!(
                        "Emergency Vehicle {} waiting at intersection {:?}",
                        veh.id, route[0]
                    );
                    let new_route = replan_route(&grid, route[0]);
                    println!(
                        "Emergency Vehicle {} replanned its route: {:?}",
                        veh.id, new_route
                    );
                    *route = new_route;
                }
            } else {
                println!(
                    "Emergency Vehicle {} reached destination at {:?}",
                    veh.id, route[0]
                );
            }
        }

        // Simulate time between ticks
        thread::sleep(Duration::from_secs(1));
    }

    println!("Simulation ended.");
}

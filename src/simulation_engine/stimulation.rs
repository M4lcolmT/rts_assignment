use crate::simulation_engine::intersections::{Intersection, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::route_generation::generate_shortest_lane_route;
use crate::simulation_engine::vehicles::{Vehicle, VehicleType};
use rand::Rng;
use std::collections::HashMap;
use std::{thread, time::Duration};

/// Spawn a vehicle at a random entry intersection, pick a random exit, and
/// generate a route (a list of lanes) to get there.
fn spawn_vehicle(
    intersections: &[Intersection],
    lanes: &[Lane],
    next_vehicle_id: &mut u64,
) -> Option<(Vehicle, Vec<Lane>)> {
    // Filter intersections to find valid entry/exit points.
    let entry_points: Vec<_> = intersections.iter().filter(|i| i.is_entry).collect();
    let exit_points: Vec<_> = intersections.iter().filter(|i| i.is_exit).collect();

    if entry_points.is_empty() || exit_points.is_empty() {
        return None;
    }

    let mut rng = rand::rng();
    let entry = entry_points[rng.random_range(0..entry_points.len())];
    let exit = exit_points[rng.random_range(0..exit_points.len())];

    // Ensure we pick distinct entry/exit intersections to avoid trivial route.
    if entry.id == exit.id {
        return None;
    }

    // Randomly choose a vehicle type.
    let vehicle_type = match rng.random_range(0..4) {
        0 => VehicleType::Car,
        1 => VehicleType::Bus,
        2 => VehicleType::Truck,
        _ => VehicleType::EmergencyVan,
    };

    // Create the vehicle.
    let vehicle = Vehicle::new(*next_vehicle_id, vehicle_type, entry.id, exit.id, 10.0);
    *next_vehicle_id += 1;

    // Generate a route in terms of lanes.
    let lane_route = generate_shortest_lane_route(lanes, entry.id, exit.id)?;

    Some((vehicle, lane_route))
}

/// Simulates vehicle movement along its lane route.
/// If the vehicle finishes its route, we remove it from the simulation.
fn simulate_vehicle_movement(vehicles: &mut Vec<(Vehicle, Vec<Lane>)>) {
    // We'll collect the IDs of vehicles that are done, then remove them afterward.
    let mut finished_vehicle_ids = Vec::new();

    for (vehicle, route) in vehicles.iter_mut() {
        // If the route is empty, it means the vehicle has already reached its destination.
        // We will mark it as finished so we can remove it after this loop.
        if route.is_empty() {
            finished_vehicle_ids.push(vehicle.id);
            continue;
        }

        // "Move" the vehicle by removing the first lane in its route.
        let current_lane = route.remove(0);

        // Print movement info.
        println!(
            "Vehicle {} is moving on lane: {} (from {:?} to {:?})",
            vehicle.id, current_lane.name, current_lane.from, current_lane.to
        );

        // If the route is now empty, that means we've arrived at the final intersection (exit).
        if route.is_empty() {
            // Mark the vehicle as finished.
            println!(
                "Vehicle {} has reached its destination at intersection: {:?}",
                vehicle.id, vehicle.exit_point
            );
            finished_vehicle_ids.push(vehicle.id);
        }
    }

    // Remove vehicles that have finished their route from the vector
    // so we do not keep printing their status.
    vehicles.retain(|(v, _)| !finished_vehicle_ids.contains(&v.id));
}

pub fn run_simulation(intersections: Vec<Intersection>, lanes: Vec<Lane>) {
    let mut vehicles: Vec<(Vehicle, Vec<Lane>)> = Vec::new();
    let mut next_vehicle_id = 1;

    // Main simulation loop.
    loop {
        // Try to spawn a vehicle with some probability or condition.
        // For demonstration, we'll spawn one on every loop iteration.
        if let Some((vehicle, route)) = spawn_vehicle(&intersections, &lanes, &mut next_vehicle_id)
        {
            // Print the route based on lane names.
            let route_names: Vec<String> = route.iter().map(|lane| lane.name.clone()).collect();
            println!(
                "Spawned vehicle {} starting from intersection {:?} to intersection {:?} with the route: {:?}",
                vehicle.id, vehicle.entry_point, vehicle.exit_point, route_names
            );
            vehicles.push((vehicle, route));
        }

        // Simulate movement for each vehicle.
        simulate_vehicle_movement(&mut vehicles);

        // Pause between simulation ticks.
        thread::sleep(Duration::from_millis(1000));
    }
}

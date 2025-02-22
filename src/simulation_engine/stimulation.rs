use crate::simulation_engine::intersections::{
    clear_intersection_for_emergency, restore_intersection, Intersection,
};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::route_generation::generate_shortest_lane_route;
use crate::simulation_engine::vehicles::{Vehicle, VehicleType};
use rand::Rng;
use std::{thread, time::Duration};

// Clear route intersections for emergency van.
fn clear_route_for_emergency(route: &[Lane], intersections: &mut [Intersection]) {
    for lane in route {
        if let Some(intersection) = intersections.iter_mut().find(|i| i.id == lane.from) {
            clear_intersection_for_emergency(intersection);
        }
    }
}

// Restore intersections once emergency van passes.
fn restore_route_intersections(intersections: &mut [Intersection], passed_lane: &Lane) {
    // Restore the intersection that the passed_lane originates from.
    if let Some(intersection) = intersections.iter_mut().find(|i| i.id == passed_lane.from) {
        restore_intersection(intersection);
    }
}

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
    // Car: 50%, Truck: 25%, Bus: 15%, EmergencyVan: 10%.
    let rand_val: f64 = rng.random_range(0.0..1.0);
    let vehicle_type = if rand_val < 0.50 {
        VehicleType::Car
    } else if rand_val < 0.75 {
        // 0.50 to 0.75 -> 25%
        VehicleType::Truck
    } else if rand_val < 0.90 {
        // 0.75 to 0.90 -> 15%
        VehicleType::Bus
    } else {
        VehicleType::EmergencyVan
    };

    // Randomize speed based on vehicle type.
    let speed = match vehicle_type {
        VehicleType::Car => rng.random_range(40.0..100.0),
        VehicleType::Bus => rng.random_range(40.0..80.0),
        VehicleType::Truck => rng.random_range(40.0..70.0),
        VehicleType::EmergencyVan => rng.random_range(60.0..120.0),
    };

    // Create the vehicle.
    let vehicle = Vehicle::new(*next_vehicle_id, vehicle_type, entry.id, exit.id, speed);
    *next_vehicle_id += 1;

    // Generate a route in terms of lanes.
    let lane_route = generate_shortest_lane_route(lanes, entry.id, exit.id)?;

    Some((vehicle, lane_route))
}

/// Simulates vehicle movement along its lane route.
/// If the vehicle finishes its route, we remove it from the simulation.
fn simulate_vehicle_movement(
    vehicles: &mut Vec<(Vehicle, Vec<Lane>)>,
    intersections: &mut [Intersection],
) {
    // We'll collect the IDs of vehicles that are done, then remove them afterward.
    let mut finished_vehicle_ids = Vec::new();

    // We'll also track lanes passed for emergency restoration.
    let mut lanes_passed = Vec::new();

    for (vehicle, route) in vehicles.iter_mut() {
        if route.is_empty() {
            finished_vehicle_ids.push(vehicle.id);
            continue;
        }

        // For emergency vans, before moving, clear the upcoming intersections.
        if vehicle.vehicle_type == VehicleType::EmergencyVan {
            clear_route_for_emergency(&route, intersections);
        }

        // "Move" the vehicle by removing the first lane in its route.
        let current_lane = route.remove(0);
        lanes_passed.push(current_lane.clone());

        println!(
            "Vehicle {:?} {} is moving on lane: {} (from {:?} to {:?})",
            vehicle.vehicle_type, vehicle.id, current_lane.name, current_lane.from, current_lane.to
        );

        // If the route is now empty, that means we've arrived at the final intersection (exit).
        if route.is_empty() {
            println!(
                "Vehicle {:?} {} has reached its destination at intersection: {:?}",
                vehicle.vehicle_type, vehicle.id, vehicle.exit_point
            );
            finished_vehicle_ids.push(vehicle.id);
        }

        // For an emergency van, once it leaves a lane, restore that intersection.
        if vehicle.vehicle_type == VehicleType::EmergencyVan {
            restore_route_intersections(intersections, &current_lane);
        }
    }

    // Remove vehicles that have finished their route.
    vehicles.retain(|(v, _)| !finished_vehicle_ids.contains(&v.id));
}

pub fn run_simulation(mut intersections: Vec<Intersection>, lanes: Vec<Lane>) {
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
                "Spawned vehicle {:?} {} starting from intersection {:?} to intersection {:?} with the route: {:?}",
                vehicle.vehicle_type, vehicle.id, vehicle.entry_point, vehicle.exit_point, route_names
            );
            vehicles.push((vehicle, route));
        }

        // Simulate movement for each vehicle.
        simulate_vehicle_movement(&mut vehicles, &mut intersections);

        // Pause between simulation ticks.
        thread::sleep(Duration::from_millis(1000));
    }
}

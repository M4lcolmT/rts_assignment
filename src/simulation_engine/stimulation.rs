use crate::control_system::traffic_light_controller::TrafficLightController;
use crate::flow_analyzer::predictive_model::{
    analyze_traffic, collect_traffic_data, predict_future_traffic, send_congestion_alerts,
};
use crate::simulation_engine::intersections::{Intersection, IntersectionControl};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::route_generation::generate_shortest_lane_route;
use crate::simulation_engine::vehicles::{Vehicle, VehicleType};
use rand::Rng;
use std::{thread, time::Duration};

// TODO: not sure if need to keep or not.
// Clear route intersections for emergency van.
// fn clear_route_for_emergency(route: &[Lane], intersections: &mut [Intersection]) {
//     for lane in route {
//         if let Some(intersection) = intersections.iter_mut().find(|i| i.id == lane.from) {
//             clear_intersection_for_emergency(intersection);
//         }
//     }
// }

// TODO: not sure if need to keep or not.
// Restore intersections once emergency van passes.
// fn restore_route_intersections(intersections: &mut [Intersection], passed_lane: &Lane) {
//     // Restore the intersection that the passed_lane originates from.
//     if let Some(intersection) = intersections.iter_mut().find(|i| i.id == passed_lane.from) {
//         restore_intersection(intersection);
//     }
// }

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
/// Now, before moving, the function checks the corresponding traffic light state:
/// 1. If the light is green, the vehicle moves and, if it was waiting, its length is removed from the lane.
/// 2. If the light is not green, the vehicle remains waiting in the lane and its length is added to the lane’s occupancy.
pub fn simulate_vehicle_movement(
    vehicles: &mut Vec<(Vehicle, Vec<Lane>)>,
    intersections: &mut [Intersection],
    lanes: &mut [Lane],
    traffic_controller: &mut TrafficLightController,
) {
    let mut finished_vehicle_ids = Vec::new();

    for (vehicle, route) in vehicles.iter_mut() {
        if route.is_empty() {
            finished_vehicle_ids.push(vehicle.id);
            continue;
        }

        // Peek the first lane in the route without removing it yet.
        let current_lane = &route[0];

        // Determine if the lane originates from a traffic-light controlled intersection.
        let intersection_opt = intersections.iter().find(|i| i.id == current_lane.from);
        let can_move = if let Some(intersection) = intersection_opt {
            if intersection.control == IntersectionControl::TrafficLight {
                // Check if the lane is currently green.
                traffic_controller.is_lane_green(intersection.id, &current_lane.name)
            } else {
                true // No traffic light control; allow immediate movement.
            }
        } else {
            true
        };

        if can_move {
            // If the vehicle was waiting, remove its length from the lane.
            // (In production code, you’d want to track waiting state per vehicle to avoid double subtraction.)
            lanes
                .iter_mut()
                .find(|lane| lane.name == current_lane.name)
                .map(|lane| lane.remove_vehicle(vehicle));

            // Vehicle moves: remove the lane from its route.
            let moving_lane = route.remove(0);
            println!(
                "Vehicle {:?} {} is moving on lane: {} (from {:?} to {:?})",
                vehicle.vehicle_type,
                vehicle.id,
                moving_lane.name,
                moving_lane.from,
                moving_lane.to
            );
            if route.is_empty() {
                println!(
                    "Vehicle {:?} {} has reached its destination at intersection: {:?}",
                    vehicle.vehicle_type, vehicle.id, vehicle.exit_point
                );
                finished_vehicle_ids.push(vehicle.id);
            }
        } else {
            // Vehicle must wait: if not already added, add its length to the lane's current_vehicle_length.
            // (A real implementation should check to avoid multiple additions per tick.)
            println!(
                "Vehicle {:?} {} is waiting at lane: {} (traffic light is red)",
                vehicle.vehicle_type, vehicle.id, current_lane.name
            );
            lanes
                .iter_mut()
                .find(|lane| lane.name == current_lane.name)
                .map(|lane| lane.add_vehicle(vehicle));
        }
    }

    vehicles.retain(|(v, _)| !finished_vehicle_ids.contains(&v.id));
}

pub fn run_simulation(mut intersections: Vec<Intersection>, mut lanes: Vec<Lane>) {
    let mut vehicles: Vec<(Vehicle, Vec<Lane>)> = Vec::new();
    let mut next_vehicle_id = 1;
    let mut traffic_controller = TrafficLightController::initialize(intersections.clone(), &lanes);

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

        traffic_controller.update_all();

        // Simulate movement for each vehicle.
        simulate_vehicle_movement(
            &mut vehicles,
            &mut intersections,
            &mut lanes,
            &mut traffic_controller,
        );

        // --- Flow Analyzer Integration ---
        // Collect traffic data (e.g., occupancy, vehicle count) from lanes, vehicles, and intersections.
        // let active_vehicles: Vec<Vehicle> = vehicles.iter().map(|(v, _)| v.clone()).collect();
        // let traffic_data = collect_traffic_data(&lanes, &active_vehicles, &intersections);

        // Analyze the traffic data for congestion hotspots.
        // let alerts = analyze_traffic(&traffic_data);
        // if !alerts.is_empty() {
        //     send_congestion_alerts(&alerts);
        // }

        // Predict future traffic conditions (e.g., for the next 10 seconds).
        // let predicted = predict_future_traffic(&traffic_data);
        // println!(
        //     "Predicted average lane occupancy: {:.2} (current: {:.2})",
        //     predicted.average_lane_occupancy, traffic_data.average_lane_occupancy
        // );

        // let congested: Vec<_> = traffic_data
        //     .intersection_congestion
        //     .iter()
        //     .filter(|&(_, &occ)| occ > 0.80)
        //     .map(|(&int_id, _)| int_id)
        //     .collect();

        // For each vehicle, attempt to generate a new "less traffic" route.
        // for (vehicle, route) in vehicles.iter_mut() {
        //     if let Some(update) = crate::flow_analyzer::predictive_model::generate_route_update(
        //         &traffic_data,
        //         route,
        //         &congested,
        //         &lanes,
        //     ) {
        //         println!("Vehicle {} route update: {}", vehicle.id, update.reason);
        //         // Replace the current route with the newly suggested route.
        //         *route = update.new_route;
        //     }
        // }

        // Predict future traffic conditions (e.g., for the next 10 seconds).
        // let predicted = predict_future_traffic(&traffic_data);
        // println!(
        //     "Predicted average lane occupancy: {:.2} (current: {:.2})",
        //     predicted.average_lane_occupancy, traffic_data.average_lane_occupancy
        // );

        // Pause between simulation ticks.
        thread::sleep(Duration::from_millis(1000));
    }
}

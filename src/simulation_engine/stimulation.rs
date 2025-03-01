use crate::control_system::traffic_light_controller::TrafficLightController;
use crate::flow_analyzer::predictive_model::{handle_accident_event, AccidentEvent};
use crate::simulation_engine::intersections::{Intersection, IntersectionControl};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::route_generation::generate_shortest_lane_route;
use crate::simulation_engine::vehicles::{Vehicle, VehicleType};
use rand::prelude::*; // brings thread_rng() and Rng methods into scope
use rand::rng;
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

        let current_lane = &route[0];
        let intersection_opt = intersections.iter().find(|i| i.id == current_lane.from);
        let mut can_move = false;

        if let Some(intersection) = intersection_opt {
            if intersection.control == IntersectionControl::TrafficLight {
                if vehicle.is_emergency {
                    // Instead of forcing only `current_lane.name`, we call
                    // the new method that also includes the opposite direction
                    traffic_controller.set_emergency_override(
                        intersection.id,
                        &current_lane.name,
                        lanes, // pass all lanes
                    );
                    can_move = true; // emergency vehicles proceed
                } else {
                    can_move =
                        traffic_controller.is_lane_green(intersection.id, &current_lane.name);
                }
            } else {
                can_move = true; // No traffic light
            }
        } else {
            can_move = true;
        }

        if can_move {
            lanes
                .iter_mut()
                .find(|ln| ln.name == current_lane.name)
                .map(|ln| ln.remove_vehicle(vehicle));

            let moving_lane = route.remove(0);
            println!(
                "Vehicle {:?} {} is moving on lane: {} (from {:?} to {:?})",
                vehicle.vehicle_type,
                vehicle.id,
                moving_lane.name,
                moving_lane.from,
                moving_lane.to
            );

            // If it’s an emergency, clear override on the intersection just left,
            // and set override on the next intersection (if any).
            if vehicle.is_emergency {
                traffic_controller.clear_emergency_override(moving_lane.from);

                if !route.is_empty() {
                    let next_lane = &route[0];
                    if let Some(next_int) = intersections.iter().find(|i| {
                        i.id == next_lane.from && i.control == IntersectionControl::TrafficLight
                    }) {
                        traffic_controller.set_emergency_override(
                            next_int.id,
                            &next_lane.name,
                            lanes, // pass all lanes again
                        );
                    }
                }
            }

            if route.is_empty() {
                println!(
                    "Vehicle {:?} {} has reached its destination at intersection: {:?}",
                    vehicle.vehicle_type, vehicle.id, vehicle.exit_point
                );
                finished_vehicle_ids.push(vehicle.id);
            }
        } else {
            println!(
                "Vehicle {:?} {} is waiting at lane: {} (traffic light is red)",
                vehicle.vehicle_type, vehicle.id, current_lane.name
            );
            lanes
                .iter_mut()
                .find(|ln| ln.name == current_lane.name)
                .map(|ln| ln.add_vehicle(vehicle));
        }
    }

    vehicles.retain(|(v, _)| !finished_vehicle_ids.contains(&v.id));
}

fn random_accident(
    intersections: &Vec<Intersection>,
    vehicles: &mut Vec<(Vehicle, Vec<Lane>)>,
    lanes: &Vec<Lane>,
) {
    // If `rng()` is available in your version:
    let mut my_rng = rng();

    // 5% chance
    if my_rng.random_bool(0.05) {
        if let Some(random_int) = intersections.choose(&mut my_rng) {
            let accident = AccidentEvent {
                intersection_id: random_int.id,
                severity: my_rng.random_range(1..=10),
            };
            println!(
                "Random accident generated at intersection {:?} with severity {}",
                accident.intersection_id, accident.severity
            );
            // Handle the accident
            handle_accident_event(&accident, vehicles, lanes);
        }
    }
}

pub fn run_simulation(mut intersections: Vec<Intersection>, mut lanes: Vec<Lane>) {
    let mut vehicles: Vec<(Vehicle, Vec<Lane>)> = Vec::new();
    let mut next_vehicle_id = 1;
    let mut traffic_controller = TrafficLightController::initialize(intersections.clone(), &lanes);

    // Create HistoricalData to store occupancy snapshots for weighted predictions.
    // Make sure you have "pub fn new(capacity: usize) -> Self" in your HistoricalData struct.
    let mut historical = crate::flow_analyzer::predictive_model::HistoricalData::new(10);

    loop {
        // === 1. Vehicle Spawning ===
        if let Some((vehicle, route)) = spawn_vehicle(&intersections, &lanes, &mut next_vehicle_id)
        {
            let route_names: Vec<String> = route.iter().map(|lane| lane.name.clone()).collect();
            println!(
                "Spawned vehicle {:?} {} from intersection {:?} to intersection {:?} with route: {:?}",
                vehicle.vehicle_type, vehicle.id, vehicle.entry_point, vehicle.exit_point, route_names
            );
            vehicles.push((vehicle, route));
        }

        // === 2. Update Traffic Lights ===
        traffic_controller.update_all();

        // === 3. Simulate Vehicle Movement ===
        simulate_vehicle_movement(
            &mut vehicles,
            &mut intersections,
            &mut lanes,
            &mut traffic_controller,
        );

        // === 4. Random Accident Generation ===
        // This function will (with a 5% chance) generate an accident and call handle_accident_event.
        random_accident(&intersections, &mut vehicles, &lanes);

        // === 5. Flow Analyzer Integration ===
        // a. Collect current traffic data (using active vehicles).
        let active_vehicles: Vec<Vehicle> = vehicles.iter().map(|(v, _)| v.clone()).collect();
        let traffic_data = crate::flow_analyzer::predictive_model::collect_traffic_data(
            &lanes,
            &active_vehicles,
            &intersections,
        );

        // b. Update historical data for weighted predictions.
        //    If your struct method is named `update_occupancy`, call that:
        historical.update(&traffic_data);

        // c. Analyze congestion and send alerts.
        let alerts = crate::flow_analyzer::predictive_model::analyze_traffic(&traffic_data);
        if !alerts.is_empty() {
            println!("--- Congestion Alerts ---");
            crate::flow_analyzer::predictive_model::send_congestion_alerts(&alerts);
        }

        // d. Predict future traffic conditions using weighted average
        let alpha = 0.8; // You can adjust this value
        let predicted = crate::flow_analyzer::predictive_model::predict_future_traffic_weighted(
            &traffic_data,
            &historical,
            alpha,
        );
        println!(
            "Predicted average lane occupancy (weighted): {:.2} (current: {:.2})",
            predicted.average_lane_occupancy, traffic_data.average_lane_occupancy
        );

        // e. Actively generate route updates for vehicles passing through congested intersections.
        //    Define congested intersections as those with occupancy > 0.80.
        let congested_intersections: Vec<_> = traffic_data
            .intersection_congestion
            .iter()
            .filter(|&(_, &occ)| occ > 0.80)
            .map(|(&int_id, _)| int_id)
            .collect();

        for (vehicle, route) in vehicles.iter_mut() {
            if let Some(route_update) =
                crate::flow_analyzer::predictive_model::generate_route_update(
                    &traffic_data,
                    route,
                    &congested_intersections,
                    &lanes,
                )
            {
                println!("Vehicle {} re-routed: {}", vehicle.id, route_update.reason);
                *route = route_update.new_route;
            }
        }

        // f. (Optional) Generate traffic light adjustment recommendations.
        let adjustments =
            crate::flow_analyzer::predictive_model::generate_signal_adjustments(&traffic_data);
        for adj in adjustments {
            println!(
                "Recommend adjusting intersection {:?}: add {} seconds green.",
                adj.intersection_id, adj.add_seconds_green
            );
            // Here you could call a method on traffic_controller to apply the adjustment.
        }

        // === 6. Pause Before Next Tick ===
        thread::sleep(Duration::from_millis(1000));
    }
}

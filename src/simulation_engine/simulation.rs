// simulation.rs

use crate::control_system::traffic_light_controller::TrafficLightController;
use crate::flow_analyzer::predictive_model::{
    analyze_traffic, collect_traffic_data, current_timestamp, generate_signal_adjustments,
    send_congestion_alerts, send_update_to_controller, TrafficUpdate,
};
use crate::simulation_engine::intersections::{Intersection, IntersectionControl, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::route_generation::generate_shortest_lane_route;
use crate::simulation_engine::vehicles::{Vehicle, VehicleType};
use crossbeam_channel::Sender;
use rand::prelude::*;
use rand::rng;
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

    // Ensure we pick distinct entry/exit intersections.
    if entry.id == exit.id {
        return None;
    }

    // Randomly choose a vehicle type.
    // Car: 50%, Truck: 25%, Bus: 15%, EmergencyVan: 10%.
    let rand_val: f64 = rng.random_range(0.0..1.0);
    let vehicle_type = if rand_val < 0.50 {
        VehicleType::Car
    } else if rand_val < 0.75 {
        VehicleType::Truck
    } else if rand_val < 0.90 {
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

    let lane_route = generate_shortest_lane_route(lanes, entry.id, exit.id)?;
    Some((vehicle, lane_route))
}

/// Simulates vehicle movement along its lane route.
pub fn simulate_vehicle_movement(
    vehicles: &mut Vec<(Vehicle, Vec<Lane>)>,
    intersections: &mut [Intersection],
    lanes: &mut [Lane],
    traffic_controller: &mut TrafficLightController,
) {
    let mut finished_vehicle_ids = Vec::new();

    // First, handle crashed vehicles and update lane accident status
    for (vehicle, route) in vehicles.iter_mut() {
        if route.is_empty() {
            continue; // Skip vehicles that have completed their route
        }

        // Get the current lane
        let current_lane = &route[0];

        // Check for accident resolution
        if vehicle.is_accident {
            if let Some(accident_time) = vehicle.accident_timestamp {
                let current_time = current_timestamp();
                let elapsed_seconds = current_time - accident_time;
                let wait_time_seconds = (vehicle.severity as u64) * 1; // 1 second per severity level

                if elapsed_seconds >= wait_time_seconds {
                    // Time to remove the accident vehicle
                    if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                        lane.remove_vehicle(&vehicle);
                        // Clear the accident flag after removal
                        lane.has_accident = false;
                        println!(
                            "Crashed Vehicle {:?} {} has been removed from: {} after {} seconds",
                            vehicle.vehicle_type, vehicle.id, lane.name, elapsed_seconds
                        );

                        // Mark the vehicle for removal from simulation
                        finished_vehicle_ids.push(vehicle.id);
                    }
                } else {
                    println!(
                        "Crashed Vehicle {:?} {} on lane: {} - Waiting for removal ({}/{} seconds)",
                        vehicle.vehicle_type,
                        vehicle.id,
                        current_lane.name,
                        elapsed_seconds,
                        wait_time_seconds
                    );
                }
            }
        }
    }

    // Then handle movement for all vehicles
    for (vehicle, route) in vehicles.iter_mut() {
        if route.is_empty() || finished_vehicle_ids.contains(&vehicle.id) {
            if !finished_vehicle_ids.contains(&vehicle.id) {
                finished_vehicle_ids.push(vehicle.id);
            }
            continue;
        }

        // Get the current lane
        let current_lane = &route[0];

        // Skip movement logic for crashed vehicles already handled above
        if vehicle.is_accident {
            continue;
        }

        // Check if the lane has an accident (any crashed vehicle)
        let lane_has_accident =
            if let Some(lane) = lanes.iter().find(|ln| ln.name == current_lane.name) {
                lane.has_accident
            } else {
                false
            };

        // If the lane has an accident, prevent all vehicles on this lane from moving
        if lane_has_accident {
            // Make sure the vehicle is registered as being in the lane
            if !vehicle.is_in_lane {
                if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                    lane.add_vehicle(&vehicle);
                }
                println!(
                    "Vehicle {:?} {} is waiting at lane: {} (lane has an accident)",
                    vehicle.vehicle_type, vehicle.id, current_lane.name
                );
                vehicle.is_in_lane = true;
            }
            continue; // Skip the rest of the movement logic
        }

        // Regular movement logic
        let intersection_opt = intersections.iter().find(|i| i.id == current_lane.from);
        let mut can_move = false;

        if let Some(intersection) = intersection_opt {
            if intersection.control == IntersectionControl::TrafficLight {
                if vehicle.is_emergency {
                    traffic_controller.set_emergency_override(
                        intersection.id,
                        &current_lane.name,
                        lanes,
                    );
                    can_move = true;
                } else {
                    can_move =
                        traffic_controller.is_lane_green(intersection.id, &current_lane.name);
                }
            } else {
                can_move = true;
            }
        } else {
            can_move = true;
        }

        // Random accident generation - vehicles only get accidents if they're moving and not already in one
        let mut my_rng = rand::rng();
        if can_move && !vehicle.is_accident && my_rng.random_bool(0.01) {
            if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                vehicle.is_accident = true;
                vehicle.severity = my_rng.random_range(1..=5);
                vehicle.accident_timestamp = Some(current_timestamp());
                lane.has_accident = true;
                println!(
                    "An accident occurred. Vehicle {:?} {} crashed on: {} with severity {}",
                    vehicle.vehicle_type, vehicle.id, lane.name, vehicle.severity
                );
                // Vehicle had an accident this tick, so it can't move anymore
                can_move = false;
            }
        }

        // Vehicle can only move if it's allowed to and not in an accident
        if can_move && !vehicle.is_accident {
            vehicle.current_lane = current_lane.name.clone();

            // Remove vehicle from lane occupancy if it was already added
            if vehicle.is_in_lane {
                if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                    lane.remove_vehicle(&vehicle);
                    vehicle.is_in_lane = false;
                }
            }

            // Remove the current lane from the route as the vehicle moves forward
            let moving_lane = route.remove(0);
            println!(
                "Vehicle {:?} {} is moving on lane: {} (from {:?} to {:?})",
                vehicle.vehicle_type,
                vehicle.id,
                moving_lane.name,
                moving_lane.from,
                moving_lane.to
            );

            // If the route is now empty, mark the vehicle as finished
            if route.is_empty() {
                println!(
                    "Vehicle {:?} {} has reached its destination at intersection: {:?}",
                    vehicle.vehicle_type, vehicle.id, vehicle.exit_point
                );
                finished_vehicle_ids.push(vehicle.id);
            }
        } else {
            // Vehicle is not moving (red light, accident, or other reason)
            if !vehicle.is_in_lane {
                if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                    lane.add_vehicle(&vehicle);
                }

                let reason = if vehicle.is_accident {
                    "vehicle is in accident"
                } else if lane_has_accident {
                    "lane has an accident"
                } else {
                    "traffic light is red"
                };

                println!(
                    "Vehicle {:?} {} is waiting at lane: {} ({})",
                    vehicle.vehicle_type, vehicle.id, current_lane.name, reason
                );
                vehicle.is_in_lane = true;
            }
        }
    }

    // Remove finished vehicles from the simulation
    vehicles.retain(|(v, _)| !finished_vehicle_ids.contains(&v.id));
}

/// Main simulation loop.
pub fn run_simulation(
    mut intersections: Vec<Intersection>,
    mut lanes: Vec<Lane>,
    tx: Sender<TrafficUpdate>,
) {
    let mut vehicles: Vec<(Vehicle, Vec<Lane>)> = Vec::new();
    let mut next_vehicle_id = 1;
    let mut traffic_controller = TrafficLightController::initialize(intersections.clone(), &lanes);

    // Create HistoricalData to store occupancy snapshots for weighted predictions.
    let mut historical = crate::flow_analyzer::predictive_model::HistoricalData::new(10);

    loop {
        // === 1. Vehicle Spawning: Spawn 6 vehicles per tick.
        for _ in 0..6 {
            if let Some((vehicle, route)) =
                spawn_vehicle(&intersections, &lanes, &mut next_vehicle_id)
            {
                let route_names: Vec<String> = route.iter().map(|lane| lane.name.clone()).collect();
                println!(
                    "Spawned vehicle {:?} {} from intersection {:?} to intersection {:?} with route: {:?}",
                    vehicle.vehicle_type, vehicle.id, vehicle.entry_point, vehicle.exit_point, route_names
                );
                vehicles.push((vehicle, route));
            }
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

        // === 4. Flow Analyzer Integration ===
        let active_vehicles: Vec<Vehicle> = vehicles.iter().map(|(v, _)| v.clone()).collect();
        let traffic_data = crate::flow_analyzer::predictive_model::collect_traffic_data(
            &lanes,
            &active_vehicles,
            &intersections,
        );
        let waiting_times: HashMap<IntersectionId, f64> = intersections
            .iter()
            .map(|intersection| (intersection.id, intersection.avg_waiting_time()))
            .collect();

        historical.update_occupancy(&traffic_data);
        historical.update_waiting_time(&waiting_times);

        let alerts = analyze_traffic(&traffic_data);
        if !alerts.is_empty() {
            send_congestion_alerts(&alerts);
        }

        let adjustments = generate_signal_adjustments(&traffic_data);
        for adj in adjustments {
            println!(
                "Recommend adjusting intersection {:?}: add {} seconds green.",
                adj.intersection_id, adj.add_seconds_green
            );
            // TODO: implement the actual adjustment logic if needed.
        }

        for (vehicle, route) in vehicles.iter_mut() {
            if let Some(route_update) =
                crate::flow_analyzer::predictive_model::generate_route_update(
                    &traffic_data,
                    route,
                    &lanes,
                    vehicle,
                )
            {
                println!("Vehicle {} re-routed: {}", vehicle.id, route_update.reason);
                *route = route_update.new_route;
            }
        }

        // (Optional) Send an update to the Traffic Light Controller.
        // TODO: verify if working or not
        let update = TrafficUpdate {
            current_data: traffic_data.clone(),
            predicted_data: crate::flow_analyzer::predictive_model::predict_future_traffic_weighted(
                &traffic_data,
                &historical,
                0.8,
            ),
            timestamp: current_timestamp(),
        };
        send_update_to_controller(update, &tx);

        // === 6. Pause Before Next Tick ===
        thread::sleep(Duration::from_millis(1000));
    }
}

// simulation.rs

use crate::control_system::traffic_light_controller::TrafficLightController;
use crate::flow_analyzer::predictive_model::{
    current_timestamp, send_update_to_controller, TrafficData, TrafficUpdate,
    collect_traffic_data, analyze_traffic, send_congestion_alerts,
    generate_signal_adjustments, handle_accident_event, AccidentEvent,
};
use crate::simulation_engine::intersections::{
    Intersection, IntersectionControl, IntersectionId,
};
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

    // Create the vehicle. (Assumes Vehicle::new also initializes fields such as 'rerouted',
    // 'waiting_logged', and 'added_to_lane' to their default false values.)
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

        if can_move {
            // If the vehicle had been added to the lane occupancy, remove it.
            if vehicle.added_to_lane {
                if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                    lane.remove_vehicle(vehicle);
                }
                vehicle.added_to_lane = false;
            }
            // Reset the waiting message flag.
            vehicle.waiting_logged = false;
            let moving_lane = route.remove(0);
            println!(
                "Vehicle {:?} {} is moving on lane: {} (from {:?} to {:?})",
                vehicle.vehicle_type, vehicle.id, moving_lane.name, moving_lane.from, moving_lane.to
            );
            if route.is_empty() {
                println!(
                    "Vehicle {:?} {} has reached its destination at intersection: {:?}",
                    vehicle.vehicle_type, vehicle.id, vehicle.exit_point
                );
                finished_vehicle_ids.push(vehicle.id);
            }
        } else {
            // When waiting, print the waiting message only once.
            if !vehicle.waiting_logged {
                println!(
                    "Vehicle {:?} {} is waiting at lane: {} (traffic light is red)",
                    vehicle.vehicle_type, vehicle.id, current_lane.name
                );
                vehicle.waiting_logged = true;
            }
            // Add vehicle occupancy only once.
            if !vehicle.added_to_lane {
                if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                    lane.add_vehicle(vehicle);
                }
                vehicle.added_to_lane = true;
            }
        }
    }

    vehicles.retain(|(v, _)| !finished_vehicle_ids.contains(&v.id));
}

/// Randomly generates an accident with a 5% chance.
/// If an accident occurs, an AccidentEvent is created and returned.
fn random_accident(
    vehicles: &mut Vec<(Vehicle, Vec<Lane>)>,
    lanes: &Vec<Lane>,
    data: &TrafficData,
) -> Option<AccidentEvent> {
    let mut my_rng = rand::rng();
    // 5% chance for an accident to occur.
    if my_rng.random_bool(0.05) {
        if let Some(random_lane) = lanes.choose(&mut my_rng) {
            let accident = AccidentEvent {
                lane: random_lane.clone(),
                severity: my_rng.random_range(1..=5),
            };
            println!(
                "Random accident generated at lane '{}' with severity {}",
                accident.lane.name, accident.severity
            );
            return Some(accident);
        }
    }
    None
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
            if let Some((vehicle, route)) = spawn_vehicle(&intersections, &lanes, &mut next_vehicle_id)
            {
                let route_names: Vec<String> =
                    route.iter().map(|lane| lane.name.clone()).collect();
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

        // === 5. Accident Handling ===
        if let Some(accident) = random_accident(&mut vehicles, &lanes, &traffic_data) {
            handle_accident_event(&accident, &mut vehicles, &lanes, &traffic_data);
        }

        // (Optional) Send an update to the Traffic Light Controller.
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

// simulation.rs

use crate::control_system::traffic_light_controller::TrafficLightController;
use crate::flow_analyzer::traffic_analyzer::{
    analyze_traffic, collect_traffic_data, current_timestamp, generate_route_update,
    generate_signal_adjustments, predict_future_traffic_weighted, send_congestion_alerts,
    HistoricalData, TrafficUpdate,
};
use crate::simulation_engine::intersections::{Intersection, IntersectionControl};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::route_generation::generate_shortest_lane_route;
use crate::simulation_engine::vehicles::{Vehicle, VehicleType};

use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
};
use rand::Rng;
use serde_json;
use std::collections::HashMap;
use std::{thread, time::Duration};

// AMIQUIP queue names
const QUEUE_TRAFFIC_DATA: &str = "traffic_data";
const QUEUE_LIGHT_ADJUSTMENTS: &str = "light_adjustments";

/// Spawn a vehicle at a random entry intersection, pick a random exit, and
/// generate a route (a list of lanes) to get there.
fn spawn_vehicle(
    intersections: &[Intersection],
    lanes: &[Lane],
    next_vehicle_id: &mut u64,
) -> Option<(Vehicle, Vec<Lane>)> {
    let entry_points: Vec<_> = intersections.iter().filter(|i| i.is_entry).collect();
    let exit_points: Vec<_> = intersections.iter().filter(|i| i.is_exit).collect();

    if entry_points.is_empty() || exit_points.is_empty() {
        return None;
    }

    let mut rng = rand::rng();
    let entry = entry_points[rng.random_range(0..entry_points.len())];
    let exit = exit_points[rng.random_range(0..exit_points.len())];

    if entry.id == exit.id {
        return None;
    }

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

    let speed = match vehicle_type {
        VehicleType::Car => rng.random_range(40.0..100.0),
        VehicleType::Bus => rng.random_range(40.0..80.0),
        VehicleType::Truck => rng.random_range(40.0..70.0),
        VehicleType::EmergencyVan => rng.random_range(60.0..120.0),
    };

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
            continue;
        }
        let current_lane = &route[0];
        if vehicle.is_accident {
            if let Some(accident_time) = vehicle.accident_timestamp {
                let current_time = current_timestamp();
                let elapsed_seconds = current_time - accident_time;
                let wait_time_seconds = (vehicle.severity as u64) * 1;

                if elapsed_seconds >= wait_time_seconds {
                    if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                        lane.remove_vehicle(&vehicle);
                        lane.has_accident = false;
                        println!(
                            "Crashed Vehicle {:?} {} has been removed from: {} after {} seconds",
                            vehicle.vehicle_type, vehicle.id, lane.name, elapsed_seconds
                        );
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

        let current_lane = &route[0];
        if vehicle.is_accident {
            continue;
        }

        let lane_has_accident =
            if let Some(lane) = lanes.iter().find(|ln| ln.name == current_lane.name) {
                lane.has_accident
            } else {
                false
            };

        if lane_has_accident {
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
            continue;
        }

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
                can_move = false;
            }
        }

        if can_move && !vehicle.is_accident {
            vehicle.current_lane = current_lane.name.clone();
            if vehicle.is_in_lane {
                if let Some(lane) = lanes.iter_mut().find(|ln| ln.name == current_lane.name) {
                    lane.remove_vehicle(&vehicle);
                    vehicle.is_in_lane = false;
                }
            }
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

    vehicles.retain(|(v, _)| !finished_vehicle_ids.contains(&v.id));
}

pub fn run_simulation(mut intersections: Vec<Intersection>, mut lanes: Vec<Lane>) {
    let mut vehicles: Vec<(Vehicle, Vec<Lane>)> = Vec::new();
    let mut next_vehicle_id = 1;
    let mut traffic_controller = TrafficLightController::initialize(intersections.clone(), &lanes);
    let mut historical = HistoricalData::new(10);

    let mut rabbit_connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")
        .expect("RabbitMQ connection");

    let publish_channel = rabbit_connection
        .open_channel(None)
        .expect("open publish channel");
    let exchange = Exchange::direct(&publish_channel);

    publish_channel
        .queue_declare(QUEUE_TRAFFIC_DATA, QueueDeclareOptions::default())
        .expect("declare traffic_data queue");

    let consumer_channel = rabbit_connection
        .open_channel(None)
        .expect("open consumer channel");

    consumer_channel
        .queue_declare(QUEUE_LIGHT_ADJUSTMENTS, QueueDeclareOptions::default())
        .expect("declare light_adjustments queue");

    // Spawn a thread to listen for LightAdjustment messages
    {
        thread::spawn(move || {
            let queue = consumer_channel
                .queue_declare(QUEUE_LIGHT_ADJUSTMENTS, QueueDeclareOptions::default())
                .expect("declare light_adjustments queue");
            let consumer = queue
                .consume(ConsumerOptions::default())
                .expect("consume light_adjustments");
            println!("Simulation: waiting for LightAdjustment messages...");

            for message in consumer.receiver() {
                match message {
                    ConsumerMessage::Delivery(delivery) => {
                        if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                            println!(
                                "[Simulation] Received LightAdjustment message: {}",
                                json_str
                            );
                            // Here you'd parse the JSON if you want to apply adjustments in real-time
                            // e.g., serde_json::from_str::<LightAdjustmentMsg>(json_str).unwrap();
                        }
                        consumer
                            .ack(delivery)
                            .expect("ack in light_adjustments consumer");
                    }
                    other => {
                        println!("Consumer ended: {:?}", other);
                        break;
                    }
                }
            }
        });
    }

    // Simulation Loop
    loop {
        // a) Spawn vehicles
        for _ in 0..6 {
            if let Some((vehicle, route)) =
                spawn_vehicle(&intersections, &lanes, &mut next_vehicle_id)
            {
                let route_names: Vec<String> = route.iter().map(|lane| lane.name.clone()).collect();
                println!(
                    "Spawned vehicle {:?} {} from {:?} to {:?} route: {:?}",
                    vehicle.vehicle_type,
                    vehicle.id,
                    vehicle.entry_point,
                    vehicle.exit_point,
                    route_names
                );
                vehicles.push((vehicle, route));
            }
        }

        // b) Update traffic lights
        traffic_controller.update_all();

        // c) Move vehicles
        simulate_vehicle_movement(
            &mut vehicles,
            &mut intersections,
            &mut lanes,
            &mut traffic_controller,
        );

        // d) Flow Analyzer logic
        let active_vehicles: Vec<Vehicle> = vehicles.iter().map(|(v, _)| v.clone()).collect();
        let traffic_data = collect_traffic_data(&lanes, &active_vehicles, &intersections);
        let waiting_times: HashMap<String, f64> = intersections
            .iter()
            .map(|i| (format!("{:?}", i.id), i.avg_waiting_time()))
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
        }

        for (vehicle, route) in vehicles.iter_mut() {
            if let Some(route_update) = generate_route_update(&traffic_data, route, &lanes, vehicle)
            {
                println!("Vehicle {} re-routed: {}", vehicle.id, route_update.reason);
                *route = route_update.new_route;
            }
        }

        // e) Send an update to the Traffic Light Controller (existing crossbeam)
        let update = TrafficUpdate {
            current_data: traffic_data.clone(),
            predicted_data: predict_future_traffic_weighted(&traffic_data, &historical, 0.8),
            timestamp: current_timestamp(),
        };

        // f) Publish traffic data to "traffic_data" queue every second.
        if let Ok(payload) = serde_json::to_vec(&update) {
            exchange
                .publish(Publish::new(&payload, QUEUE_TRAFFIC_DATA))
                .expect("publish traffic_data");
        } else {
            println!("ERROR serializing update");
        }

        // g) Sleep
        thread::sleep(Duration::from_millis(1000));
    }
}

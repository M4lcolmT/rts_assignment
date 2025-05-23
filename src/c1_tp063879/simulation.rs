// simulation.rs
use crate::c1_tp063879::intersections::{Intersection, IntersectionControl};
use crate::c1_tp063879::lanes::Lane;
use crate::c1_tp063879::route_generation::generate_shortest_lane_route;
use crate::c1_tp063879::vehicles::{Vehicle, VehicleType};
use crate::c3_tp063987::traffic_light_controller::TrafficLightController;
use crate::global_variables::{AMQP_URL, QUEUE_TRAFFIC_DATA};
use crate::shared_data::current_timestamp;
use crate::shared_data::{TrafficData, TrafficUpdate, VehicleData};

use amiquip::{Connection, Exchange, Publish, QueueDeclareOptions};
use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

// Collect current traffic data from lanes and intersections, including vehicle data.
pub fn collect_traffic_data(
    lanes: &[Lane],
    intersections: &[Intersection],
    vehicle_data: Vec<VehicleData>,
) -> TrafficData {
    let mut lane_occupancy = HashMap::new();
    for lane in lanes {
        let occupancy = lane.current_vehicle_length / lane.length_meters;
        lane_occupancy.insert(lane.name.clone(), occupancy);
    }

    let mut accident_lanes = HashSet::new();
    for lane in lanes {
        if lane.has_accident {
            accident_lanes.insert(lane.name.clone());
        }
    }

    let mut intersection_congestion = HashMap::new();
    for intersection in intersections {
        let outgoing: Vec<_> = lanes.iter().filter(|l| l.from == intersection.id).collect();
        if outgoing.is_empty() {
            intersection_congestion.insert(format!("{:?}", intersection.id), 0.0);
        } else {
            let sum_occ: f64 = outgoing
                .iter()
                .map(|l| l.current_vehicle_length / l.length_meters)
                .sum();
            let avg = sum_occ / outgoing.len() as f64;
            intersection_congestion.insert(format!("{:?}", intersection.id), avg);
        }
    }

    let mut intersection_waiting_time = HashMap::new();
    for intersection in intersections {
        let outgoing: Vec<_> = lanes.iter().filter(|l| l.from == intersection.id).collect();
        if outgoing.is_empty() {
            intersection_waiting_time.insert(format!("{:?}", intersection.id), 0.0);
        } else {
            let total_waiting: f64 = outgoing.iter().map(|l| l.waiting_time).sum();
            let avg_waiting = total_waiting / outgoing.len() as f64;
            intersection_waiting_time.insert(format!("{:?}", intersection.id), avg_waiting);
        }
    }

    TrafficData {
        lane_occupancy,
        accident_lanes,
        intersection_congestion,
        intersection_waiting_time,
        vehicle_data,
    }
}

// Helper function to simulate rush hour by adjusting the number of vehicles spawned per iteration.
// The spawn count increases linearly from 2 to 5 during the first 20 seconds of a 40-second cycle,
// then decreases from 5 to 2 over the next 20 seconds, and repeats.
fn simulate_rush_hour(simulation_start: u64, current_time: u64) -> usize {
    let elapsed = current_time - simulation_start; // elapsed time in seconds
    let cycle_duration = 40; // full cycle length in seconds
    let cycle_time = elapsed % cycle_duration;
    let spawn_count = if cycle_time <= 20 {
        // Increasing phase: 2 to 5 vehicles.
        let ratio = cycle_time as f64 / 20.0;
        2.0 + ratio * 3.0 // 2 + (0 to 3) = 2 to 5
    } else {
        // Decreasing phase: 5 to 2 vehicles.
        let ratio = (cycle_time - 20) as f64 / 20.0;
        5.0 - ratio * 3.0 // 5 - (0 to 3) = 5 to 2
    };
    spawn_count.round() as usize
}

// Helper function to check if a vehicle is overspeeding.
// For each vehicle type, we consider the vehicle to be overspeeding if its speed is in the upper 10% of its spawn range.
fn verify_speed_limit(vehicle: &Vehicle) -> bool {
    match vehicle.vehicle_type {
        VehicleType::Car => {
            // Car speed range: 80.0 - 140.0; threshold = 80 + 0.90*(60) = 134.0
            vehicle.speed >= 134.0
        }
        VehicleType::Bus => {
            // Bus speed range: 70.0 - 100.0; threshold = 70 + 0.90*(30) = 97.0
            vehicle.speed >= 97.0
        }
        VehicleType::Truck => {
            // Truck speed range: 60.0 - 90.0; threshold = 60 + 0.90*(30) = 87.0
            vehicle.speed >= 87.0
        }
        VehicleType::EmergencyVan => {
            // EmergencyVan speed range: 120.0 - 180.0; threshold = 120 + 0.90*(60) = 174.0
            vehicle.speed >= 174.0
        }
    }
}

// Spawns a new vehicle and computes its route based on predicted traffic data.
pub fn spawn_vehicle(
    intersections: &Arc<Mutex<Vec<Intersection>>>,
    lanes: &Arc<Mutex<Vec<Lane>>>,
    current_traffic_data: &TrafficData,
    next_vehicle_id: &mut u64,
) -> Option<(Vehicle, Vec<Lane>)> {
    let intersections_guard = intersections.lock().unwrap();
    let lanes_guard = lanes.lock().unwrap();
    let entry_points: Vec<_> = intersections_guard.iter().filter(|i| i.is_entry).collect();
    let exit_points: Vec<_> = intersections_guard.iter().filter(|i| i.is_exit).collect();

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
    let vehicle_type = if rand_val < 0.70 {
        VehicleType::Car
    } else if rand_val < 0.90 {
        VehicleType::Truck
    } else if rand_val < 0.99 {
        VehicleType::Bus
    } else {
        VehicleType::EmergencyVan
    };

    let speed = match vehicle_type {
        VehicleType::Car => rng.random_range(80.0..140.0),
        VehicleType::Bus => rng.random_range(70.0..100.0),
        VehicleType::Truck => rng.random_range(60.0..90.0),
        VehicleType::EmergencyVan => rng.random_range(120.0..180.0),
    };

    let vehicle = Vehicle::new(*next_vehicle_id, vehicle_type, entry.id, exit.id, speed);
    *next_vehicle_id += 1;

    let entry_id = entry.id;
    let exit_id = exit.id;
    drop(intersections_guard);

    let filtered_lanes: Vec<Lane> = lanes_guard
        .clone()
        .into_iter()
        .filter(|lane| {
            if lane.has_accident {
                return false;
            }
            if let Some(&occ) = current_traffic_data.lane_occupancy.get(&lane.name) {
                if occ > 0.75 {
                    return false;
                }
            }
            true
        })
        .collect();

    let route = generate_shortest_lane_route(&filtered_lanes, entry_id, exit_id)?;
    Some((vehicle, route))
}

// Simulates a vehicle’s journey as an independent async task.
// The vehicle pushes its event data into the shared vehicle_events vector when it reaches its destination or crashes.
pub async fn simulate_vehicle_journey(
    mut vehicle: Vehicle,
    mut route: Vec<Lane>,
    intersections: Arc<Mutex<Vec<Intersection>>>,
    lanes: Arc<Mutex<Vec<Lane>>>,
    traffic_controller: Arc<Mutex<TrafficLightController>>,
    active_ids: Arc<Mutex<HashSet<u64>>>,
    vehicle_events: Arc<Mutex<Vec<VehicleData>>>,
) {
    let mut rng = SmallRng::seed_from_u64(1);
    while let Some(current_lane) = route.first() {
        let mut add_success = false;
        {
            let mut lanes_guard = lanes.lock().unwrap();
            if let Some(lane) = lanes_guard.iter_mut().find(|l| l.name == current_lane.name) {
                if lane.add_vehicle(&vehicle) {
                    vehicle.is_in_lane = true;
                    add_success = true;
                } else {
                    println!(
                        "Vehicle {:?} {} could not be added to lane {} (capacity full). Retrying...",
                        vehicle.vehicle_type, vehicle.id, lane.name
                    );
                }
            }
        }
        if !add_success {
            sleep(Duration::from_secs_f64(5.0)).await;
            continue;
        }

        let lane_has_accident = {
            let lanes_guard = lanes.lock().unwrap();
            lanes_guard
                .iter()
                .find(|l| l.name == current_lane.name)
                .map(|l| l.has_accident)
                .unwrap_or(false)
        };
        if lane_has_accident {
            let accident_severity = if vehicle.is_accident {
                vehicle.severity
            } else {
                2
            };
            let target_wait = accident_severity as f64 * 1.5;
            if vehicle.waiting_start.is_none() {
                vehicle.waiting_start = Some(current_timestamp());
            }
            let waited = current_timestamp() - vehicle.waiting_start.unwrap();
            if (waited as f64) < target_wait {
                let remaining = target_wait - waited as f64;
                println!(
                    "Vehicle {:?} {} waiting at lane {} due to accident. Waiting {:.2} more seconds.",
                    vehicle.vehicle_type, vehicle.id, current_lane.name, remaining
                );
                sleep(Duration::from_secs_f64(remaining)).await;
                let total_waited = current_timestamp() - vehicle.waiting_start.unwrap();
                vehicle.waiting_time += total_waited;
                vehicle.waiting_start = None;
            }
        }

        let intersection_opt = {
            let intersections_guard = intersections.lock().unwrap();
            intersections_guard
                .iter()
                .find(|i| i.id == current_lane.from)
                .cloned()
        };
        if let Some(intersection) = intersection_opt {
            if intersection.control == IntersectionControl::TrafficLight {
                let can_move = {
                    let tc = traffic_controller.lock().unwrap();
                    tc.is_lane_green(intersection.id, &current_lane.name)
                };
                if !can_move {
                    if vehicle.is_emergency() {
                        {
                            let mut tc = traffic_controller.lock().unwrap();
                            tc.set_emergency_override_route(
                                intersection.id,
                                vec![current_lane.name.clone()],
                            );
                        }
                        println!(
                            "Emergency vehicle {:?} {} triggered override at intersection {:?} on lane {}.",
                            vehicle.vehicle_type, vehicle.id, intersection.id, current_lane.name
                        );
                    } else {
                        if vehicle.waiting_start.is_none() {
                            vehicle.waiting_start = Some(current_timestamp());
                        }
                        let remaining_phase = {
                            let tc = traffic_controller.lock().unwrap();
                            if let Some(ctrl) = tc.controllers.get(&intersection.id) {
                                ctrl.phases[ctrl.current_phase_index]
                                    .duration
                                    .saturating_sub(ctrl.elapsed_in_phase)
                            } else {
                                1
                            }
                        };
                        sleep(Duration::from_secs(remaining_phase)).await;
                        continue;
                    }
                } else {
                    if let Some(start) = vehicle.waiting_start {
                        let waited = current_timestamp() - start;
                        vehicle.waiting_time += waited;
                        vehicle.waiting_start = None;
                    }
                }
            }
        }

        let accident_probability = if verify_speed_limit(&vehicle) {
            0.15
        } else {
            0.10
        };
        if rng.random_bool(accident_probability) {
            let crashed_timestamp = current_timestamp();
            vehicle.accident_timestamp = Some(crashed_timestamp);
            let crash_severity = rng.random_range(1..=3);
            vehicle.severity = crash_severity;
            let crash_wait = crash_severity as f64 * 1.5;
            println!(
                "Vehicle {:?} {} crashed on lane {} with severity {} at {:?}. Waiting {:.2} seconds before removal.",
                vehicle.vehicle_type, vehicle.id, current_lane.name, crash_severity, vehicle.accident_timestamp, crash_wait
            );
            sleep(Duration::from_secs_f64(crash_wait)).await;
            println!(
                "Vehicle {:?} {} removed from simulation due to crash.",
                vehicle.vehicle_type, vehicle.id
            );
            {
                let mut lanes_guard = lanes.lock().unwrap();
                if let Some(lane) = lanes_guard.iter_mut().find(|l| l.name == current_lane.name) {
                    lane.remove_vehicle(&vehicle);
                }
            }
            {
                let mut active = active_ids.lock().unwrap();
                active.remove(&vehicle.id);
            }
            {
                let mut veh_ev = vehicle_events.lock().unwrap();
                veh_ev.push(VehicleData {
                    id: vehicle.id,
                    waiting_time: vehicle.waiting_time,
                    accident_timestamp: vehicle.accident_timestamp,
                    severity: vehicle.severity,
                    current_lane: current_lane.name.to_string(),
                });
            }
            return;
        }

        let travel_time_secs = current_lane.length_meters / vehicle.speed;
        println!(
            "Vehicle {:?} {} traveling lane {} (from {:?} to {:?}) in {:.2} seconds.",
            vehicle.vehicle_type,
            vehicle.id,
            current_lane.name,
            current_lane.from,
            current_lane.to,
            travel_time_secs
        );
        sleep(Duration::from_secs_f64(travel_time_secs)).await;
        vehicle.waiting_start = None;
        {
            let mut lanes_guard = lanes.lock().unwrap();
            if let Some(lane) = lanes_guard.iter().position(|l| l.name == current_lane.name) {
                lanes_guard.get_mut(lane).unwrap().remove_vehicle(&vehicle);
            }
        }
        route.remove(0);
    }
    println!(
        "Vehicle {:?} {} reached destination. Total waiting time: {} seconds.",
        vehicle.vehicle_type, vehicle.id, vehicle.waiting_time
    );
    {
        let mut veh_ev = vehicle_events.lock().unwrap();
        veh_ev.push(VehicleData {
            id: vehicle.id,
            waiting_time: vehicle.waiting_time,
            accident_timestamp: vehicle.accident_timestamp,
            severity: vehicle.severity,
            current_lane: "".to_string(),
        });
    }
    {
        let mut active = active_ids.lock().unwrap();
        active.remove(&vehicle.id);
    }
}

pub async fn run_simulation(
    intersections: Arc<Mutex<Vec<Intersection>>>,
    lanes: Arc<Mutex<Vec<Lane>>>,
) {
    // Record simulation start time.
    let simulation_start = current_timestamp();

    // Initialize the traffic light controller.
    let (iclones, lclones) = {
        let intersections_guard = intersections.lock().unwrap();
        let lanes_guard = lanes.lock().unwrap();
        (intersections_guard.clone(), lanes_guard.clone())
    };
    let tc = TrafficLightController::initialize(iclones, &lclones);
    let traffic_controller = Arc::new(Mutex::new(tc));

    // Spawn the traffic light update loop as a concurrent task.
    tokio::spawn(TrafficLightController::run_update_loop(Arc::clone(
        &traffic_controller,
    )));

    let mut next_vehicle_id = 1;
    let active_ids: Arc<Mutex<HashSet<u64>>> = Arc::new(Mutex::new(HashSet::new()));
    let vehicle_events: Arc<Mutex<Vec<VehicleData>>> = Arc::new(Mutex::new(vec![]));

    let mut rabbit_connection = Connection::insecure_open(AMQP_URL).expect("RabbitMQ connection");
    let publish_channel = rabbit_connection
        .open_channel(None)
        .expect("open publish channel");
    let exchange = Exchange::direct(&publish_channel);
    publish_channel
        .queue_declare(QUEUE_TRAFFIC_DATA, QueueDeclareOptions::default())
        .expect("declare traffic_data queue");

    loop {
        // Calculate dynamic spawn count based on rush hour simulation.
        let current_time = current_timestamp();
        let spawn_count = simulate_rush_hour(simulation_start, current_time);
        println!(
            "Elapsed time: {} sec - Spawning {} vehicle(s) this iteration.",
            current_time - simulation_start,
            spawn_count
        );

        // Take snapshots of lanes and intersections.
        let (lanes_snapshot, intersections_snapshot) = {
            let lanes_guard = lanes.lock().unwrap();
            let intersections_guard = intersections.lock().unwrap();
            (lanes_guard.clone(), intersections_guard.clone())
        };
        // Extract vehicle events and clear the shared vector.
        let vehicle_data_snapshot = {
            let mut veh_ev = vehicle_events.lock().unwrap();
            let data = veh_ev.clone();
            veh_ev.clear();
            data
        };
        let current_traffic_data = collect_traffic_data(
            &lanes_snapshot,
            &intersections_snapshot,
            vehicle_data_snapshot,
        );

        // Spawn a batch of vehicles concurrently.
        for _ in 0..spawn_count {
            if let Some((vehicle, route)) = spawn_vehicle(
                &intersections,
                &lanes,
                &current_traffic_data,
                &mut next_vehicle_id,
            ) {
                {
                    let mut active = active_ids.lock().unwrap();
                    if active.contains(&vehicle.id) {
                        continue;
                    }
                    active.insert(vehicle.id);
                }
                println!(
                    "Spawned vehicle {:?} {} from {:?} to {:?}. Route: {:?}",
                    vehicle.vehicle_type,
                    vehicle.id,
                    vehicle.entry_point,
                    vehicle.exit_point,
                    route.iter().map(|l| l.name.clone()).collect::<Vec<_>>()
                );
                let intersections_clone = Arc::clone(&intersections);
                let lanes_clone = Arc::clone(&lanes);
                let tc_clone = Arc::clone(&traffic_controller);
                let active_ids_clone = Arc::clone(&active_ids);
                let vehicle_events_clone = Arc::clone(&vehicle_events);
                tokio::spawn(simulate_vehicle_journey(
                    vehicle,
                    route,
                    intersections_clone,
                    lanes_clone,
                    tc_clone,
                    active_ids_clone,
                    vehicle_events_clone,
                ));
            }
        }

        let update = TrafficUpdate {
            current_data: current_traffic_data,
            timestamp: current_timestamp(),
        };
        match serde_json::to_vec(&update) {
            Ok(payload) => {
                if let Err(err) = exchange.publish(Publish::new(&payload, QUEUE_TRAFFIC_DATA)) {
                    println!("Error publishing traffic_data: {}", err);
                }
            }
            Err(err) => {
                println!("Error serializing update: {}", err);
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}

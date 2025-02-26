mod simulation_engine {
    pub mod stimulation {
        use crate::control_system::traffic_light_controller::TrafficLightController;
        use crate::flow_analyzer::{analyze_traffic, predict_future_traffic, send_congestion_alerts};
        use crate::flow_analyzer::predictive_model::TrafficData;
        use crate::simulation_engine::intersections::{clear_intersection_for_emergency, restore_intersection, Intersection};
        use crate::simulation_engine::lanes::Lane;
        use crate::simulation_engine::route_generation::generate_shortest_lane_route;
        use crate::simulation_engine::vehicles::{Vehicle, VehicleType};
        use rand::Rng;
        use std::{collections::HashMap, thread, time::Duration};

        const DWELL_TICKS: u32 = 2;

        pub fn collect_traffic_data(lanes: &[Lane], vehicles: &[Vehicle], intersections: &[Intersection]) -> TrafficData {
            let total_vehicles = vehicles.len();
            let mut total_occupancy = 0.0;
            for lane in lanes {
                let occ = lane.current_vehicle_length / lane.length_meters;
                total_occupancy += occ;
            }
            let average_lane_occupancy = if lanes.is_empty() { 0.0 } else { total_occupancy / lanes.len() as f64 };

            let mut intersection_congestion = HashMap::new();
            for intersection in intersections {
                let outgoing: Vec<_> = lanes.iter().filter(|l| l.from == intersection.id).collect();
                if outgoing.is_empty() {
                    intersection_congestion.insert(intersection.id, 0.0);
                } else {
                    let sum_occ: f64 = outgoing.iter().map(|l| l.current_vehicle_length / l.length_meters).sum();
                    let avg = sum_occ / outgoing.len() as f64;
                    intersection_congestion.insert(intersection.id, avg);
                }
            }
            TrafficData { total_vehicles, average_lane_occupancy, intersection_congestion }
        }

        pub fn spawn_vehicle(intersections: &[Intersection], lanes: &mut Vec<Lane>, next_vehicle_id: &mut u64) -> Option<(Vehicle, Vec<Lane>, u32)> {
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
            if let Some(first_lane) = lane_route.first() {
                if let Some(lane) = lanes.iter_mut().find(|l| l.name == first_lane.name) {
                    if !lane.add_vehicle(&vehicle) {
                        return None;
                    }
                }
            }
            Some((vehicle, lane_route, 0))
        }

        pub fn simulate_vehicle_movement(vehicles: &mut Vec<(Vehicle, Vec<Lane>, u32)>, intersections: &mut Vec<Intersection>, lanes: &mut Vec<Lane>, controller: &TrafficLightController) {
            let mut finished_vehicle_ids = Vec::new();
            for (vehicle, route, dwell) in vehicles.iter_mut() {
                if route.is_empty() {
                    finished_vehicle_ids.push(vehicle.id);
                    continue;
                }
                if *dwell < DWELL_TICKS {
                    *dwell += 1;
                    continue;
                }
                *dwell = 0;
                if vehicle.vehicle_type == VehicleType::EmergencyVan {
                    if let Some(first_lane) = route.first() {
                        controller.set_emergency_override(first_lane.from, true, false);
                    }
                }
                let current_lane = route.first().unwrap().clone();
                if let Some(lane) = lanes.iter_mut().find(|l| l.name == current_lane.name) {
                    lane.remove_vehicle(vehicle);
                }
                route.remove(0);
                if let Some(next_lane) = route.first() {
                    if let Some(lane) = lanes.iter_mut().find(|l| l.name == next_lane.name) {
                        lane.add_vehicle(vehicle);
                    }
                }
                println!("Vehicle {:?} {} moved from lane {} (from {:?} to {:?})", vehicle.vehicle_type, vehicle.id, current_lane.name, current_lane.from, current_lane.to);
                if route.is_empty() {
                    println!("Vehicle {:?} {} reached destination at {:?}", vehicle.vehicle_type, vehicle.id, vehicle.exit_point);
                    finished_vehicle_ids.push(vehicle.id);
                }
                if vehicle.vehicle_type == VehicleType::EmergencyVan {
                    controller.clear_emergency_override(current_lane.from);
                }
            }
            vehicles.retain(|(v, _, _)| !finished_vehicle_ids.contains(&v.id));
        }

        pub fn run_simulation(intersections: Vec<Intersection>, mut lanes: Vec<Lane>, controller: &TrafficLightController) {
            let mut vehicles: Vec<(Vehicle, Vec<Lane>, u32)> = Vec::new();
            let mut next_vehicle_id = 1;
            let mut intersections = intersections;
            loop {
                if let Some((vehicle, route, _)) = spawn_vehicle(&intersections, &mut lanes, &mut next_vehicle_id) {
                    let route_names: Vec<String> = route.iter().map(|lane| lane.name.clone()).collect();
                    println!("Spawned vehicle {:?} {} from {:?} to {:?} with route: {:?}", vehicle.vehicle_type, vehicle.id, vehicle.entry_point, vehicle.exit_point, route_names);
                    vehicles.push((vehicle, route, 0));
                }
                simulate_vehicle_movement(&mut vehicles, &mut intersections, &mut lanes, controller);
                let active_vehicles: Vec<crate::simulation_engine::vehicles::Vehicle> = vehicles.iter().map(|(v,_,_)| v.clone()).collect();
                let traffic_data = collect_traffic_data(&lanes, &active_vehicles, &intersections);
                let alerts = analyze_traffic(&traffic_data);
                if !alerts.is_empty() {
                    send_congestion_alerts(&alerts);
                }
                let predicted = predict_future_traffic(&traffic_data);
                println!("Predicted average lane occupancy: {:.2} (current: {:.2})", predicted.average_lane_occupancy, traffic_data.average_lane_occupancy);
                let congested: Vec<_> = traffic_data.intersection_congestion.iter().filter(|&(_, &occ)| occ > 0.80).map(|(&int_id, _)| int_id).collect();
                for (vehicle, route, _) in vehicles.iter_mut() {
                    if let Some(update) = crate::flow_analyzer::predictive_model::generate_route_update(&traffic_data, route, &congested, &lanes) {
                        println!("Vehicle {} route update: {}", vehicle.id, update.reason);
                        *route = update.new_route;
                    }
                }
                let predicted = predict_future_traffic(&traffic_data);
                println!("Predicted average lane occupancy: {:.2} (current: {:.2})", predicted.average_lane_occupancy, traffic_data.average_lane_occupancy);
                thread::sleep(Duration::from_millis(1000));
            }
        }
    }
}
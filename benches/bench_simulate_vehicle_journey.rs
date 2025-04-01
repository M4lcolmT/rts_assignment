use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio;

use rts_assignment::control_system::traffic_light_controller::TrafficLightController;
use rts_assignment::simulation_engine::intersections::create_intersections;
use rts_assignment::simulation_engine::lanes::create_lanes;
use rts_assignment::simulation_engine::route_generation::generate_shortest_lane_route;
use rts_assignment::simulation_engine::simulation::simulate_vehicle_journey;
use rts_assignment::simulation_engine::vehicles::{Vehicle, VehicleType};

fn bench_simulate_vehicle_journey(c: &mut Criterion) {
    let intersections = Arc::new(Mutex::new(create_intersections()));
    let lanes = Arc::new(Mutex::new(create_lanes()));

    // Initialize traffic light controller
    let tc_instance = {
        let intersections_snapshot = intersections.lock().unwrap().clone();
        let lanes_snapshot = lanes.lock().unwrap().clone();
        TrafficLightController::initialize(intersections_snapshot, &lanes_snapshot)
    };
    let traffic_controller = Arc::new(Mutex::new(tc_instance));

    // Spawn the traffic light controller update loop
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.spawn(TrafficLightController::run_update_loop(Arc::clone(
        &traffic_controller,
    )));

    let active_ids = Arc::new(Mutex::new(HashSet::new()));
    let vehicle_events = Arc::new(Mutex::new(Vec::new()));

    let mut group = c.benchmark_group("simulate_vehicle_journey");
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("simulate_vehicle_journey", |b| {
        b.iter_custom(|_iters| {
            let start = std::time::Instant::now();
            rt.block_on(async {
                // Lock intersections and lanes to generate a random route.
                let intersections_guard = intersections.lock().unwrap();
                let lanes_guard = lanes.lock().unwrap();
                let entry_points: Vec<_> =
                    intersections_guard.iter().filter(|i| i.is_entry).collect();
                let exit_points: Vec<_> =
                    intersections_guard.iter().filter(|i| i.is_exit).collect();

                let mut rng = rand::rng();
                let entry = entry_points[rng.random_range(0..entry_points.len())];
                let exit = exit_points[rng.random_range(0..exit_points.len())];
                let entry_id = entry.id;
                let exit_id = exit.id;

                let route = generate_shortest_lane_route(&lanes_guard, entry_id, exit_id)
                    .expect("Route should be found");

                println!(
                    "\nVehicle Route: {}",
                    route
                        .iter()
                        .map(|l| l.name.clone())
                        .collect::<Vec<_>>()
                        .join(" -> ")
                );

                drop(intersections_guard);
                drop(lanes_guard);

                let intersections_clone = Arc::clone(&intersections);
                let lanes_clone = Arc::clone(&lanes);
                let tc_clone = Arc::clone(&traffic_controller);
                let active_ids_clone = Arc::clone(&active_ids);
                let vehicle_events_clone = Arc::clone(&vehicle_events);

                // Create a single vehicle for simplicity.
                let speed = rng.random_range(80.0..140.0);

                let vehicle = Vehicle::new(1, VehicleType::Car, entry_id, exit_id, speed);
                simulate_vehicle_journey(
                    vehicle,
                    route,
                    intersections_clone,
                    lanes_clone,
                    tc_clone,
                    active_ids_clone,
                    vehicle_events_clone,
                )
                .await;
            });
            start.elapsed()
        })
    });
    group.finish();
}

criterion_group!(benches, bench_simulate_vehicle_journey);
criterion_main!(benches);

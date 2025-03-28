// simulation_main.rs
use rts_assignment::simulation_engine::intersections::create_intersections;
use rts_assignment::simulation_engine::lanes::create_lanes;
use rts_assignment::simulation_engine::simulation::run_simulation;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    env_logger::init();

    let intersections = Arc::new(Mutex::new(create_intersections()));
    let lanes = Arc::new(Mutex::new(create_lanes()));

    run_simulation(intersections, lanes).await;
}

// simulation_main.rs
use rts_assignment::c1_tp063879::intersections::create_intersections;
use rts_assignment::c1_tp063879::lanes::create_lanes;
use rts_assignment::c1_tp063879::simulation::run_simulation;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    env_logger::init();

    let intersections = Arc::new(Mutex::new(create_intersections()));
    let lanes = Arc::new(Mutex::new(create_lanes()));

    run_simulation(intersections, lanes).await;
}

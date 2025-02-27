mod control_system;
mod flow_analyzer;
mod monitoring;
mod simulation_engine;

use simulation_engine::intersections::create_intersections;
use simulation_engine::lanes::create_lanes;
use simulation_engine::stimulation;
use std::thread;
use std::time::Duration;

fn main() {
    let intersections = create_intersections();
    let lanes = create_lanes();

    stimulation::run_simulation(intersections, lanes);
    loop {
        thread::sleep(Duration::from_secs(5));
    }
}

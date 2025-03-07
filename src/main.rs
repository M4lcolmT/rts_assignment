mod control_system;
mod flow_analyzer;
mod monitoring;
mod simulation_engine;

use crossbeam_channel::unbounded;
use flow_analyzer::traffic_analyzer::TrafficUpdate;
use simulation_engine::intersections::create_intersections;
use simulation_engine::lanes::create_lanes;
use simulation_engine::simulation;
use std::thread;

fn main() {
    // TODO: see how to incorporate logging into component 4.
    env_logger::init();
    let intersections = create_intersections();
    let lanes = create_lanes();

    // Create a channel for sending traffic updates.
    let (tx, rx) = unbounded::<TrafficUpdate>();

    thread::spawn(move || {
        // This simulates the Traffic Light Controller or the System Monitoring component.
        for update in rx.iter() {
            log::info!(
                "[Controller] Received traffic update at timestamp {}: {:#?}",
                update.timestamp,
                update
            );
        }
    });
    simulation::run_simulation(intersections, lanes, tx);
}

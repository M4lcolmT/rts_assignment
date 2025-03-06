mod control_system;
mod flow_analyzer;
mod monitoring;
mod simulation_engine;

use crossbeam_channel::unbounded;
use flow_analyzer::predictive_model::TrafficUpdate;
use simulation_engine::intersections::create_intersections;
use simulation_engine::lanes::create_lanes;
use simulation_engine::simulation;
use std::thread;
use std::time::Duration;

fn main() {
    env_logger::init();
    let intersections = create_intersections();
    let lanes = create_lanes();

    // Create a channel for sending traffic updates.
    let (tx, rx) = unbounded::<TrafficUpdate>();

    // Spawn a thread to receive and process updates.
    thread::spawn(move || {
        // This simulates the Traffic Light Controller or the System Monitoring component.
        for update in rx.iter() {
            log::info!(
                "[Controller] Received traffic update at timestamp {}: {:#?}",
                update.timestamp,
                update
            );
            // Here you can add code to adjust traffic light timings based on the update.
        }
    });

    simulation::run_simulation(intersections, lanes, tx);
    loop {
        thread::sleep(Duration::from_secs(5));
    }
}

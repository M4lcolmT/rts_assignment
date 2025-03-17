use rts_assignment::simulation_engine::intersections::create_intersections;
use rts_assignment::simulation_engine::lanes::create_lanes;
use rts_assignment::simulation_engine::simulation::run_simulation;

fn main() {
    env_logger::init();

    let intersections = create_intersections();
    let lanes = create_lanes();

    // Crossbeam channel for sending traffic updates (if you still need it)
    // let (tx, rx) = unbounded::<TrafficUpdate>();

    // // (Optional) spawn a thread to listen for updates from the simulation
    // std::thread::spawn(move || {
    //     for update in rx {
    //         log::info!(
    //             "[Simulation MAIN] Received traffic update at timestamp {}: {:#?}",
    //             update.timestamp,
    //             update
    //         );
    //     }
    // });

    // Now run the simulation
    run_simulation(intersections, lanes);
}

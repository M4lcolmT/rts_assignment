mod flow_analyzer;
mod simulation_engine;
mod control_system;
mod flow_analyzer;
mod monitoring;

use simulation_engine::intersections::create_intersections;
use simulation_engine::lanes::create_lanes;
use simulation_engine::stimulation;
use control_system::traffic_light_controller::TrafficLightController;
use std::thread;
use std::time::Duration;

fn main() {
    // 1. Initialize the simulation engine components
    let mut intersections = create_intersections(); // needs to be mutable now
    let lanes = create_lanes();

    // 2. Initialize the TrafficLightController, passing in necessary data.
    let controller = TrafficLightController::new(intersections.clone()); // Pass a clone to avoid ownership issues

    // 3. Start the TrafficLightController's thread
    controller.run();

    // 4. Start the main simulation loop (in the simulation engine).
    //    This might need to be adjusted based on how `stimulation::run_simulation` works.
    //    The controller and the simulation engine should run concurrently.
    stimulation::run_simulation(intersections, lanes);

    // Keep the main thread alive (e.g., using a loop)
    loop {
        thread::sleep(Duration::from_secs(5));
    }
}

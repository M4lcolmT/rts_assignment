use simulation_engine::intersections::create_intersections;
use simulation_engine::lanes::create_lanes;
use simulation_engine::stimulation;
use control_system::traffic_light_controller::TrafficLightController;
use std::{thread, time::Duration};

fn main() {
    let intersections = create_intersections();
    let lanes = create_lanes();
    let controller = TrafficLightController::new(intersections.clone());
    controller.run();
    stimulation::run_simulation(intersections, lanes, &controller);
    loop {
        thread::sleep(Duration::from_secs(5));
    }
}
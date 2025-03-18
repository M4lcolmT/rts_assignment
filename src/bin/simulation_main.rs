use rts_assignment::simulation_engine::intersections::create_intersections;
use rts_assignment::simulation_engine::lanes::create_lanes;
use rts_assignment::simulation_engine::simulation::run_simulation;

fn main() {
    env_logger::init();

    let intersections = create_intersections();
    let lanes = create_lanes();

    run_simulation(intersections, lanes);
}

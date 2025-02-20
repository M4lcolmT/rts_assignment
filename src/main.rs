mod simulation_engine;

use simulation_engine::intersections::create_intersections;
use simulation_engine::lanes::create_lanes;
use simulation_engine::stimulation::run_simulation;

fn main() {
    let intersections = create_intersections();
    let lanes = create_lanes();

    // You might need to initialize the connected intersections for each Intersection here.
    // For example, by scanning through your lanes and adding adjacent IDs to each intersection's connected vector.

    run_simulation(intersections, lanes);
}

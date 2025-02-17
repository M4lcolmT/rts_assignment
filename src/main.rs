mod simulation_engine;

fn main() {
    // Now we can call the function we re-exported in mod.rs
    simulation_engine::run_simulation();
}

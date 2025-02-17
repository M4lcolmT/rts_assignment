// simulation_engine/mod.rs
pub mod events;
pub mod grid;
pub mod intersection;
pub mod lane;
pub mod movement;
pub mod route_generation;
pub mod vehicles;

// Treat main.rs as a module called `engine_main` (or just `main`).
pub mod main;

// Re-export the public function from main.rs, so top-level can do
// `simulation_engine::run_simulation()`.
pub use main::run_simulation;

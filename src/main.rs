mod communication;
mod engine;
mod models;

use communication::messages::SimulationMessage;
use engine::simulation::SimulationEngine;
use rand::Rng;
use std::error::Error;
use tokio;
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create an mpsc channel for simulation messages.
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);

    // Initialize the simulation engine with the message sender.
    let mut simulation = SimulationEngine::new(tx.clone());

    // Create two intervals:
    // - simulation_interval (100 ms): for vehicle spawning and simulation updates.
    // - update_interval (1 second): for printing real-time traffic updates.
    let mut simulation_interval = interval(Duration::from_millis(100));
    let mut update_interval = interval(Duration::from_secs(1));

    // Spawn a task to act as the "Traffic Flow Analyzer" that handles simulation messages.
    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            match message {
                SimulationMessage::VehicleSpawned(vehicle) => {
                    println!("Vehicle spawned: {}", vehicle.vehicle_type);
                }
                SimulationMessage::VehicleMoved {
                    vehicle_id,
                    from,
                    to,
                } => {
                    println!(
                        "Vehicle {} moved from intersection {} to {}",
                        vehicle_id, from, to
                    );
                }
                SimulationMessage::TrafficLightChanged {
                    intersection_id,
                    is_green,
                } => {
                    println!(
                        "Intersection {} traffic light changed to {}",
                        intersection_id,
                        if is_green { "green" } else { "red" }
                    );
                }
                SimulationMessage::IntersectionCongested {
                    intersection_id,
                    load,
                } => {
                    println!(
                        "Intersection {} is congested: {:.0}% of capacity",
                        intersection_id,
                        load * 100.0
                    );
                }
                SimulationMessage::SimulationTick(time) => {
                    println!("Simulation tick: {:.1} seconds", time);
                }
            }
        }
    });

    let mut rng = rand::rng();

    // Run the simulation loop indefinitely.
    loop {
        tokio::select! {
            // Every 100 ms: update simulation state.
            _ = simulation_interval.tick() => {
                // With a 30% chance, spawn a new vehicle (arrival event).
                if rng.random::<f32>() < 0.3 {
                    simulation.spawn_vehicle().await?;
                }

                // Update the simulation (moves vehicles along their routes).
                simulation.update().await?;

                // Check each intersection for congestion.
                for (id, intersection) in simulation.intersections.iter() {
                    let load = intersection.current_vehicles.len() as f32 / intersection.max_capacity as f32;
                    if load > 0.8 {
                        simulation.message_sender.send(
                            SimulationMessage::IntersectionCongested {
                                intersection_id: *id,
                                load,
                            }
                        ).await?;
                    }
                }

                // Send a simulation tick message (includes the simulation time).
                simulation.message_sender.send(
                    SimulationMessage::SimulationTick(simulation.simulation_time)
                ).await?;
            },

            // Every 1 second: print a real-time traffic update.
            _ = update_interval.tick() => {
                let vehicle_count = simulation.vehicles.len();
                let total_waiting: f32 = simulation.vehicles.values()
                    .map(|v| v.waiting_time)
                    .sum();
                let avg_waiting = if vehicle_count > 0 {
                    total_waiting / vehicle_count as f32
                } else {
                    0.0
                };

                println!(
                    "Traffic Update: {} vehicles, Average waiting time: {:.2} seconds",
                    vehicle_count, avg_waiting
                );
            }
        }
    }
}

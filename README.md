# 🚦 Real-Time Traffic Simulation System

A **Rust-based real-time traffic simulation** that models vehicle flow, intersections, and traffic lights - powered by **concurrency**, **RabbitMQ messaging**, and **dynamic route updates**.


## 🧩 Overview

This project simulates an **intelligent traffic system** where vehicles move through intersections managed by adaptive traffic lights.  
It integrates with **RabbitMQ** to enable real-time communication between simulation components (e.g., flow analyzers and traffic controllers).

The system continuously:
- Spawns vehicles at random entry points  
- Updates traffic lights dynamically  
- Detects and resolves congestion or accidents  
- Publishes live traffic data  
- Listens for signal adjustment commands  

Built as part of a **Real-Time Systems Assignment**.


## ⚙️ Key Features

- 🚘 **Vehicle Simulation**  
  Generates different vehicle types (Car, Bus, Truck, Emergency Van) with random speeds and routes.

- 🚦 **Dynamic Traffic Light Control**  
  Each intersection uses a `TrafficLightController` that adjusts signals based on real-time data.

- 💥 **Accident Simulation**  
  Randomized crash events occur during simulation; lanes become blocked until cleared.

- 📊 **Flow Analysis & Prediction**  
  Tracks waiting times and lane occupancy; predicts future congestion using weighted historical data.

- 🔄 **Route Recalculation**  
  Vehicles can be re-routed when congestion or accidents occur.

- 📨 **RabbitMQ Integration**  
  - Publishes `traffic_data` updates to other components  
  - Consumes `light_adjustments` messages from controllers  
  - Enables scalable, distributed simulation behavior

- ⏱️ **Real-Time Loop**  
  The simulation runs in continuous 1-second intervals, updating all vehicle and intersection states concurrently.


## 💬 RabbitMQ Queues

| Queue Name | Description |
|-------------|-------------|
| `traffic_data` | Publishes real-time traffic data (vehicle states, congestion, predictions) |
| `light_adjustments` | Receives external traffic light timing updates or commands |


## 🧩 Main Components

| Module | Description |
|--------|-------------|
| `simulation.rs` | Core simulation loop (spawning, movement, accidents, messaging) |
| `control_system/traffic_light_controller.rs` | Manages signal timing per intersection |
| `flow_analyzer/traffic_analyzer.rs` | Collects and analyzes traffic patterns |
| `simulation_engine/*` | Defines intersections, lanes, vehicles, and route generation logic |


## 🧰 Technologies Used

| Category | Tools / Libraries |
|-----------|------------------|
| **Language** | Rust |
| **Concurrency** | `crossbeam_channel`, `thread` |
| **Messaging** | `amiquip` (RabbitMQ client) |
| **Serialization** | `serde`, `serde_json` |
| **Randomization** | `rand` |
| **Data Structures** | `HashMap`, custom structs |
| **Real-Time Logic** | Looped simulation with 1-second intervals |


## 🚀 How It Works

1. **Vehicle Spawn**  
   Randomly generates vehicles at entry intersections with unique routes.

2. **Traffic Light Updates**  
   Traffic lights update via the `TrafficLightController`, which can receive adjustments from RabbitMQ.

3. **Vehicle Movement & Accidents**  
   Vehicles move based on light status and can occasionally crash, blocking a lane for a certain duration.

4. **Data Collection**  
   Real-time metrics (lane occupancy, waiting time) are analyzed and stored as historical data.

5. **Prediction & Alerts**  
   The system predicts future congestion and generates signal adjustments.

6. **RabbitMQ Messaging**  
   - Publishes `TrafficUpdate` messages (`traffic_data` queue)  
   - Consumes `LightAdjustment` messages to modify lights dynamically  

7. **Loop**  
   Repeats every second for continuous, live simulation.


## 🧪 Example Output (Console)

```bash
Spawned vehicle Car 12 from A to C route: [A→B, B→C]
Vehicle Car 12 is moving on lane: A→B
An accident occurred. Vehicle Truck 5 crashed on: B→C with severity 3
Recommend adjusting intersection B: add 4 seconds green.
Published traffic update to RabbitMQ: traffic_data
[Simulation] Received LightAdjustment message: {"intersection_id":2,"add_seconds_green":5}

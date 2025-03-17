// traffic_analyzer.rs

use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::simulation_engine::intersections::{Intersection, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::route_generation::generate_shortest_lane_route;
use crate::simulation_engine::vehicles::Vehicle;

// === AMIQIP ADDITION ===
use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result as AmiquipResult,
};
use serde_json;
use std::thread;

/// We can reuse your data structs for publishing/consuming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficData {
    pub total_vehicles: usize,
    pub lane_occupancy: HashMap<String, f64>,
    pub accident_lanes: HashSet<String>,
    pub intersection_congestion: HashMap<IntersectionId, f64>,
    pub intersection_waiting_time: HashMap<IntersectionId, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficUpdate {
    pub current_data: TrafficData,
    pub predicted_data: TrafficData,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionAlert {
    pub intersection: Option<IntersectionId>,
    pub message: String,
    pub recommended_action: String,
}

#[derive(Debug)]
pub struct HistoricalData {
    pub capacity: usize,
    pub occupancy_history: HashMap<IntersectionId, VecDeque<f64>>,
    pub waiting_time_history: HashMap<IntersectionId, VecDeque<f64>>,
}

impl HistoricalData {
    /// Create a new HistoricalData with a given capacity (e.g., 10 snapshots).
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            occupancy_history: HashMap::new(),
            waiting_time_history: HashMap::new(),
        }
    }

    pub fn update_occupancy(&mut self, data: &TrafficData) {
        for (&int_id, &occ) in &data.intersection_congestion {
            let deque = self
                .occupancy_history
                .entry(int_id)
                .or_insert_with(VecDeque::new);
            if deque.len() == self.capacity {
                deque.pop_front();
            }
            deque.push_back(occ);
        }
    }

    pub fn update_waiting_time(&mut self, waiting_times: &HashMap<IntersectionId, f64>) {
        for (&int_id, &wt) in waiting_times {
            let deque = self
                .waiting_time_history
                .entry(int_id)
                .or_insert_with(VecDeque::new);
            if deque.len() == self.capacity {
                deque.pop_front();
            }
            deque.push_back(wt);
        }
    }

    pub fn average_occupancy_for(&self, int_id: IntersectionId) -> f64 {
        if let Some(deque) = self.occupancy_history.get(&int_id) {
            if !deque.is_empty() {
                let sum: f64 = deque.iter().sum();
                return sum / deque.len() as f64;
            }
        }
        0.0
    }

    pub fn average_waiting_time_for(&self, int_id: IntersectionId) -> f64 {
        if let Some(deque) = self.waiting_time_history.get(&int_id) {
            if !deque.is_empty() {
                let sum: f64 = deque.iter().sum();
                return sum / deque.len() as f64;
            }
        }
        0.0
    }
}

pub fn collect_traffic_data(
    lanes: &[Lane],
    vehicles: &[Vehicle],
    intersections: &[Intersection],
) -> TrafficData {
    let total_vehicles = vehicles.len();

    // Compute per-lane occupancy.
    let mut lane_occupancy = HashMap::new();
    for lane in lanes {
        // Calculate occupancy for each lane.
        let occupancy = lane.current_vehicle_length / lane.length_meters;
        lane_occupancy.insert(lane.name.to_string(), occupancy);
    }

    let mut accident_lanes = HashSet::new();
    for lane in lanes {
        if lane.has_accident {
            accident_lanes.insert(lane.name.to_string());
        }
    }

    // Compute intersection-level congestion (average occupancy of outgoing lanes).
    // This data is for calculating whether to increase/decrease traffic light timings.
    let mut intersection_congestion = HashMap::new();
    for intersection in intersections {
        let outgoing: Vec<_> = lanes.iter().filter(|l| l.from == intersection.id).collect();
        if outgoing.is_empty() {
            intersection_congestion.insert(intersection.id, 0.0);
        } else {
            let sum_occ: f64 = outgoing
                .iter()
                .map(|l| l.current_vehicle_length / l.length_meters)
                .sum();
            let avg = sum_occ / outgoing.len() as f64;
            intersection_congestion.insert(intersection.id, avg);
        }
    }

    let mut intersection_waiting_time = HashMap::new();
    for intersection in intersections {
        let outgoing: Vec<_> = lanes.iter().filter(|l| l.from == intersection.id).collect();
        if outgoing.is_empty() {
            intersection_waiting_time.insert(intersection.id, 0.0);
        } else {
            let total_waiting: f64 = outgoing.iter().map(|l| l.waiting_time).sum();
            let avg_waiting = total_waiting / outgoing.len() as f64;
            intersection_waiting_time.insert(intersection.id, avg_waiting);
        }
    }

    TrafficData {
        total_vehicles,
        lane_occupancy,
        accident_lanes,
        intersection_congestion,
        intersection_waiting_time,
    }
}

pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn send_update_to_controller(update: TrafficUpdate, tx: &Sender<TrafficUpdate>) {
    // [ORIGINAL LOGIC UNCHANGED]
    if let Err(e) = tx.send(update) {
        log::info!("Failed to send traffic update to controller: {}", e);
    }
}

pub fn analyze_traffic(data: &TrafficData) -> Vec<CongestionAlert> {
    // [ORIGINAL LOGIC UNCHANGED]
    let mut alerts = Vec::new();
    for (lane_name, &occupancy) in &data.lane_occupancy {
        if occupancy > 0.75 {
            alerts.push(CongestionAlert {
                intersection: None,
                message: format!(
                    "Lane '{}' is heavily congested (occupancy: {:.2})",
                    lane_name, occupancy
                ),
                recommended_action: String::from("Vehicles on-route to the lane will be rerouted."),
            });
        }
    }
    for (&int_id, &cong) in &data.intersection_congestion {
        if cong > 0.80 {
            alerts.push(CongestionAlert {
                intersection: Some(int_id),
                message: format!(
                    "Intersection {:?} is heavily congested ({:.2})",
                    int_id, cong
                ),
                recommended_action: String::from(
                    "Adjust traffic light timings to avoid congestion.",
                ),
            });
        }
    }
    alerts
}

/// Predict future traffic conditions using a weighted average that combines current data
/// with historical data. `alpha` is the weight for current data (e.g., 0.8).
pub fn predict_future_traffic_weighted(
    data: &TrafficData,
    historical: &HistoricalData,
    alpha: f64,
) -> TrafficData {
    let mut new_congestion = HashMap::new();
    let mut new_waiting_time = HashMap::new();

    // Compute weighted occupancy for each intersection.
    for (&int_id, &current_occ) in &data.intersection_congestion {
        let hist_occ = historical.average_occupancy_for(int_id);
        let predicted_occ = alpha * current_occ + (1.0 - alpha) * hist_occ;
        new_congestion.insert(int_id, predicted_occ.min(1.0));

        log::info!(
            "[Prediction] Intersection {:?}: current occupancy = {:.2}, historical average = {:.2}, predicted occupancy = {:.2}",
            int_id, current_occ, hist_occ, predicted_occ
        );
    }

    // Compute weighted waiting time for each intersection.
    for (&int_id, &current_wait) in &data.intersection_waiting_time {
        let hist_wait = historical.average_waiting_time_for(int_id);
        let predicted_wait = alpha * current_wait + (1.0 - alpha) * hist_wait;
        new_waiting_time.insert(int_id, predicted_wait);

        log::info!(
            "[Prediction] Intersection {:?}: current waiting time = {:.2}, historical average = {:.2}, predicted waiting time = {:.2}",
            int_id, current_wait, hist_wait, predicted_wait
        );
    }

    TrafficData {
        total_vehicles: data.total_vehicles,
        lane_occupancy: data.lane_occupancy.clone(),
        accident_lanes: data.accident_lanes.clone(),
        intersection_congestion: new_congestion,
        intersection_waiting_time: new_waiting_time,
    }
}

pub fn send_congestion_alerts(alerts: &[CongestionAlert]) {
    // [ORIGINAL LOGIC UNCHANGED: just prints them]
    for alert in alerts {
        println!("--- Congestion Alert ---");
        if let Some(int_id) = alert.intersection {
            println!("Affected Intersection: {:?}", int_id);
        }
        println!("Message: {}", alert.message);
        println!("Recommended Action: {}", alert.recommended_action);
    }
}

#[derive(Debug, Clone)]
pub struct RouteUpdate {
    pub new_route: Vec<Lane>,
    pub reason: String,
}

/// If one or more lanes in the current route are congested, attempt to generate a new route
/// that avoids those congested lanes and any lane with an accident.
/// The caller should update the vehicle's route with the returned RouteUpdate.
pub fn generate_route_update(
    traffic_data: &TrafficData,
    current_route: &[Lane],
    all_lanes: &[Lane],
    vehicle: &mut Vehicle,
) -> Option<RouteUpdate> {
    // Only attempt a reroute if the vehicle hasn't been rerouted already.
    if vehicle.rerouted {
        return None;
    }

    // Determine the current lane based on whether the vehicle is in accident mode.
    let (current_lane, current_lane_index) = if vehicle.is_accident {
        // Find the index of the lane that matches the vehicle's current_lane.
        if let Some(index) = current_route
            .iter()
            .position(|lane| lane.name == vehicle.current_lane)
        {
            (&current_route[index], index)
        } else {
            return None;
        }
    } else {
        // Use the first lane in the current route.
        (current_route.first()?, 0)
    };

    // If the vehicle is already in an accident lane, do not attempt a reroute.
    if traffic_data.accident_lanes.contains(&current_lane.name) {
        println!(
            "Vehicle {:?} {}: Already in an accident lane {}, no rerouting.",
            vehicle.vehicle_type, vehicle.id, current_lane.name
        );
        return None;
    }

    let occupancy_threshold = 0.75;

    // Identify congested lanes in the current route.
    let congested_lanes: Vec<&Lane> = current_route
        .iter()
        .filter(|lane| {
            traffic_data
                .lane_occupancy
                .get(&lane.name)
                .map(|&occ| occ > occupancy_threshold)
                .unwrap_or(false)
        })
        .collect();

    // Identify accident lanes:
    // - If vehicle is in accident mode, only consider lanes after the current lane.
    // - Otherwise, consider all lanes in the route.
    let accident_lanes_in_route: Vec<&Lane> = if vehicle.is_accident {
        current_route
            .iter()
            .skip(current_lane_index + 1)
            .filter(|lane| traffic_data.accident_lanes.contains(&lane.name))
            .collect()
    } else {
        current_route
            .iter()
            .filter(|lane| traffic_data.accident_lanes.contains(&lane.name))
            .collect()
    };

    // If no congested lanes and no accident lanes are detected, no update is needed.
    if congested_lanes.is_empty() && accident_lanes_in_route.is_empty() {
        return None;
    }

    // Log detected congestions and accidents.
    if !congested_lanes.is_empty() {
        println!(
            "Vehicle {:?} {}: Congested lanes detected in current route: {:?}",
            vehicle.vehicle_type,
            vehicle.id,
            congested_lanes
                .iter()
                .map(|lane| lane.name.as_str())
                .collect::<Vec<_>>()
        );
    }
    if !accident_lanes_in_route.is_empty() {
        println!(
            "Vehicle {:?} {}: Accident lanes detected in the {} route: {:?}",
            vehicle.vehicle_type,
            vehicle.id,
            if vehicle.is_accident {
                "remaining"
            } else {
                "current"
            },
            accident_lanes_in_route
                .iter()
                .map(|lane| lane.name.as_str())
                .collect::<Vec<_>>()
        );
    }

    // Determine start and target intersections from the current route.
    let current_intersection = current_lane.from;
    let target_intersection = current_route.last()?.to;

    // Filter out lanes that are congested or have accidents.
    let filtered_lanes: Vec<Lane> = all_lanes
        .iter()
        .filter(|lane| {
            let occupancy_ok = traffic_data
                .lane_occupancy
                .get(&lane.name)
                .map(|&occ| occ <= occupancy_threshold)
                .unwrap_or(true);
            let no_accident = !traffic_data.accident_lanes.contains(&lane.name);
            occupancy_ok && no_accident
        })
        .cloned()
        .collect();

    // Generate a new route using the filtered lanes.
    if let Some(new_route) =
        generate_shortest_lane_route(&filtered_lanes, current_intersection, target_intersection)
    {
        println!(
            "Vehicle {:?} {}: New route generated: {:?}",
            vehicle.vehicle_type,
            vehicle.id,
            new_route
                .iter()
                .map(|lane| lane.name.as_str())
                .collect::<Vec<_>>()
        );
        // Mark the vehicle as having been rerouted.
        vehicle.rerouted = true;
        let mut reason = String::new();
        if !congested_lanes.is_empty() {
            reason.push_str(&format!(
                "Congested lanes (occupancy > {:.2}) detected; ",
                occupancy_threshold
            ));
        }
        if !accident_lanes_in_route.is_empty() {
            reason.push_str("Accident lanes detected; ");
        }
        reason.push_str("rerouting suggested to avoid these segments.");
        return Some(RouteUpdate { new_route, reason });
    }

    None
}

#[derive(Debug, Clone)]
pub struct SignalAdjustment {
    pub intersection_id: IntersectionId,
    pub add_seconds_green: u32,
}

pub fn generate_signal_adjustments(data: &TrafficData) -> Vec<SignalAdjustment> {
    // [ORIGINAL LOGIC UNCHANGED]
    let threshold = 0.80;
    let mut adjustments = Vec::new();
    for (&int_id, &occ) in &data.intersection_congestion {
        if occ > threshold {
            adjustments.push(SignalAdjustment {
                intersection_id: int_id,
                add_seconds_green: 10,
            });
        }
    }
    adjustments
}

/// ========== AMIQIP DEMO SECTION ==========
/// This function runs a loop that consumes from the "traffic_data" queue
/// and processes each TrafficUpdate. Then it optionally publishes alerts or
/// recommended actions to other queues.
pub fn start_analyzer_rabbitmq() -> AmiquipResult<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
    let channel = connection.open_channel(None)?;

    // Declare or get the queue we consume from
    let queue = channel.queue_declare("traffic_data", QueueDeclareOptions::default())?;
    let consumer = queue.consume(ConsumerOptions::default())?;
    println!("[Analyzer] Waiting for TrafficUpdate messages on 'traffic_data'...");

    // We can also prepare an exchange or queue for publishing alerts
    let exchange = Exchange::direct(&channel);
    channel.queue_declare("congestion_alerts", QueueDeclareOptions::default())?;
    // Or we might also publish to "light_adjustments" if we want to directly
    // send recommended actions to the traffic controller.

    // In a real system, you'd keep a HistoricalData instance here to do predictions, etc.
    // For brevity, we’ll just do basic congestion analysis on each message.
    for message in consumer.receiver() {
        match message {
            ConsumerMessage::Delivery(delivery) => {
                if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                    if let Ok(update) = serde_json::from_str::<TrafficUpdate>(json_str) {
                        println!("[Analyzer] Got TrafficUpdate: {:?}", update);

                        // 1) Analyze the current_data
                        let alerts = analyze_traffic(&update.current_data);
                        if !alerts.is_empty() {
                            // 2) For demonstration, publish each alert as JSON to "congestion_alerts"
                            for alert in &alerts {
                                if let Ok(alert_json) = serde_json::to_string(alert) {
                                    exchange.publish(Publish::new(
                                        alert_json.as_bytes(),
                                        "congestion_alerts",
                                    ))?;
                                }
                            }
                            println!(
                                "[Analyzer] Published {} congestion alerts to 'congestion_alerts'",
                                alerts.len()
                            );
                        }
                    }
                }
                consumer.ack(delivery)?;
            }
            other => {
                println!("[Analyzer] Consumer ended: {:?}", other);
                break;
            }
        }
    }

    connection.close()
}

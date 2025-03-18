// traffic_analyzer.rs
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::simulation_engine::intersections::Intersection;
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::route_generation::generate_shortest_lane_route;
use crate::simulation_engine::vehicles::Vehicle;

// === AMIQIP ADDITION ===
use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result as AmiquipResult,
};

/// Modified TrafficData: the intersection-related HashMaps now use String keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficData {
    pub total_vehicles: usize,
    pub lane_occupancy: HashMap<String, f64>,
    pub accident_lanes: HashSet<String>,
    pub intersection_congestion: HashMap<String, f64>,
    pub intersection_waiting_time: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficUpdate {
    pub current_data: TrafficData,
    pub predicted_data: TrafficData,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CongestionAlert {
    pub intersection: Option<String>,
    pub message: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalData {
    pub capacity: usize,
    pub occupancy_history: HashMap<String, VecDeque<f64>>,
    pub waiting_time_history: HashMap<String, VecDeque<f64>>,
}

impl HistoricalData {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            occupancy_history: HashMap::new(),
            waiting_time_history: HashMap::new(),
        }
    }

    pub fn update_occupancy(&mut self, data: &TrafficData) {
        for (int_id, &occ) in &data.intersection_congestion {
            let deque = self
                .occupancy_history
                .entry(int_id.clone())
                .or_insert_with(VecDeque::new);
            if deque.len() == self.capacity {
                deque.pop_front();
            }
            deque.push_back(occ);
        }
    }

    pub fn update_waiting_time(&mut self, waiting_times: &HashMap<String, f64>) {
        for (int_id, &wt) in waiting_times {
            let deque = self
                .waiting_time_history
                .entry(int_id.clone())
                .or_insert_with(VecDeque::new);
            if deque.len() == self.capacity {
                deque.pop_front();
            }
            deque.push_back(wt);
        }
    }

    pub fn average_occupancy_for(&self, int_id: &str) -> f64 {
        if let Some(deque) = self.occupancy_history.get(int_id) {
            if !deque.is_empty() {
                let sum: f64 = deque.iter().sum();
                return sum / deque.len() as f64;
            }
        }
        0.0
    }

    pub fn average_waiting_time_for(&self, int_id: &str) -> f64 {
        if let Some(deque) = self.waiting_time_history.get(int_id) {
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
    let mut intersection_congestion = HashMap::new();
    for intersection in intersections {
        let outgoing: Vec<_> = lanes.iter().filter(|l| l.from == intersection.id).collect();
        if outgoing.is_empty() {
            intersection_congestion.insert(format!("{:?}", intersection.id), 0.0);
        } else {
            let sum_occ: f64 = outgoing
                .iter()
                .map(|l| l.current_vehicle_length / l.length_meters)
                .sum();
            let avg = sum_occ / outgoing.len() as f64;
            intersection_congestion.insert(format!("{:?}", intersection.id), avg);
        }
    }

    let mut intersection_waiting_time = HashMap::new();
    for intersection in intersections {
        let outgoing: Vec<_> = lanes.iter().filter(|l| l.from == intersection.id).collect();
        if outgoing.is_empty() {
            intersection_waiting_time.insert(format!("{:?}", intersection.id), 0.0);
        } else {
            let total_waiting: f64 = outgoing.iter().map(|l| l.waiting_time).sum();
            let avg_waiting = total_waiting / outgoing.len() as f64;
            intersection_waiting_time.insert(format!("{:?}", intersection.id), avg_waiting);
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

pub fn analyze_traffic(data: &TrafficData) -> Vec<CongestionAlert> {
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
    for (int_id, &cong) in &data.intersection_congestion {
        if cong > 0.80 {
            alerts.push(CongestionAlert {
                intersection: Some(int_id.clone()), // now int_id is a String
                message: format!("Intersection {} is heavily congested ({:.2})", int_id, cong),
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
    for (int_id, &current_occ) in &data.intersection_congestion {
        let hist_occ = historical.average_occupancy_for(int_id);
        let predicted_occ = alpha * current_occ + (1.0 - alpha) * hist_occ;
        new_congestion.insert(int_id.clone(), predicted_occ.min(1.0));
    }

    // Compute weighted waiting time for each intersection.
    for (int_id, &current_wait) in &data.intersection_waiting_time {
        let hist_wait = historical.average_waiting_time_for(int_id);
        let predicted_wait = alpha * current_wait + (1.0 - alpha) * hist_wait;
        new_waiting_time.insert(int_id.clone(), predicted_wait);
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
    for alert in alerts {
        println!("--- Congestion Alert ---");
        if let Some(ref int_id) = alert.intersection {
            println!("Affected Intersection: {}", int_id);
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
        if let Some(index) = current_route
            .iter()
            .position(|lane| lane.name == vehicle.current_lane)
        {
            (&current_route[index], index)
        } else {
            return None;
        }
    } else {
        (current_route.first()?, 0)
    };

    if traffic_data.accident_lanes.contains(&current_lane.name) {
        println!(
            "Vehicle {:?} {}: Already in an accident lane {}, no rerouting.",
            vehicle.vehicle_type, vehicle.id, current_lane.name
        );
        return None;
    }

    let occupancy_threshold = 0.75;
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

    if congested_lanes.is_empty() && accident_lanes_in_route.is_empty() {
        return None;
    }

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

    let current_intersection = current_lane.from;
    let target_intersection = current_route.last()?.to;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalAdjustment {
    pub intersection_id: String,
    pub add_seconds_green: u32,
}

pub fn generate_signal_adjustments(data: &TrafficData) -> Vec<SignalAdjustment> {
    let threshold = 0.80;
    let mut adjustments = Vec::new();
    for (int_id, &occ) in &data.intersection_congestion {
        if occ > threshold {
            adjustments.push(SignalAdjustment {
                intersection_id: int_id.clone(),
                add_seconds_green: 10,
            });
        }
    }
    adjustments
}

// === AMIQIP ===
// Listen for traffic data and publish to "congestion_alerts" queue.
pub fn start_analyzer_rabbitmq() -> AmiquipResult<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
    let channel = connection.open_channel(None)?;

    let exchange = Exchange::direct(&channel);

    let traffic_data_queue =
        channel.queue_declare("traffic_data", QueueDeclareOptions::default())?;

    let consumer = traffic_data_queue.consume(ConsumerOptions::default())?;
    println!("[Analyzer] Waiting for TrafficUpdate on 'traffic_data'...");

    channel.queue_declare("congestion_alerts", QueueDeclareOptions::default())?;

    for message in consumer.receiver() {
        println!("received message from simulation to flow analyzer");
        match message {
            ConsumerMessage::Delivery(delivery) => {
                if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                    if let Ok(update) = serde_json::from_str::<TrafficUpdate>(json_str) {
                        let alerts = analyze_traffic(&update.current_data);
                        if !alerts.is_empty() {
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

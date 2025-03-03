use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::simulation_engine::intersections::{Intersection, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::vehicles::Vehicle;

#[derive(Debug, Clone)]
pub struct AccidentEvent {
    pub intersection_id: IntersectionId,
    pub severity: u8, // or any other fields you need
}

pub fn handle_accident_event(
    accident: &AccidentEvent,
    vehicles: &mut Vec<(Vehicle, Vec<Lane>)>,
    lanes: &[Lane],
) {
    println!(
        "[FlowAnalyzer] Accident at intersection {:?} (severity: {})",
        accident.intersection_id, accident.severity
    );

    // For example, if any vehicle's route includes a lane that leads to
    // the accident intersection, we attempt to re-route it:
    for (vehicle, route) in vehicles.iter_mut() {
        // Check if the route eventually goes to the accident intersection
        let heading_to_accident = route.iter().any(|lane| lane.to == accident.intersection_id);
        if heading_to_accident {
            println!(
                "[FlowAnalyzer] Vehicle {} is affected by accident; re-routing.",
                vehicle.id
            );

            // The simplest approach is to see where the vehicle is right now:
            if let Some(current_lane) = route.first() {
                let current_intersection = current_lane.from;
                // Attempt to generate a new route that avoids the accident intersection.
                // (One approach is to call your route_generation with a “blacklist” of intersections to avoid.)
                if let Some(new_route) =
                    crate::simulation_engine::route_generation::generate_shortest_lane_route(
                        lanes,
                        current_intersection,
                        vehicle.exit_point, // or pick some other logic
                    )
                {
                    // Replace the vehicle’s route with the new route
                    *route = new_route;
                    println!(
                        "[FlowAnalyzer] Vehicle {} rerouted successfully.",
                        vehicle.id
                    );
                }
            }
        }
    }
}

/// Aggregated traffic data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]

pub struct TrafficData {
    pub total_vehicles: usize,
    pub average_lane_occupancy: f64,
    pub intersection_congestion: HashMap<IntersectionId, f64>,
    // New field for average waiting times per intersection.
    pub intersection_waiting_time: HashMap<IntersectionId, f64>,
}

/// A congestion alert with a message and recommended action.
#[derive(Debug, Clone)]
pub struct CongestionAlert {
    pub intersection: Option<IntersectionId>,
    pub message: String,
    pub recommended_action: String,
}

/// ---------------- NEW: HistoricalData for Weighted Predictions ----------------

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

    /// Update occupancy history for each intersection with the latest occupancy value.
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

    /// Update waiting time history using the current waiting times.
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

    /// Compute the average occupancy for a given intersection from historical data.
    pub fn average_occupancy_for(&self, int_id: IntersectionId) -> f64 {
        if let Some(deque) = self.occupancy_history.get(&int_id) {
            if !deque.is_empty() {
                let sum: f64 = deque.iter().sum();
                return sum / deque.len() as f64;
            }
        }
        0.0
    }

    /// Compute the average waiting time for a given intersection from historical data.
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

/// ------------------------------------------------------------------------------

/// Collect real-time data from lanes, vehicles, and intersections.
pub fn collect_traffic_data(
    lanes: &[Lane],
    vehicles: &[Vehicle],
    intersections: &[Intersection],
) -> TrafficData {
    let total_vehicles = vehicles.len();

    // Compute average lane occupancy.
    let mut total_occupancy = 0.0;
    for lane in lanes {
        let occ = lane.current_vehicle_length / lane.length_meters;
        total_occupancy += occ;
    }
    let average_lane_occupancy = if lanes.is_empty() {
        0.0
    } else {
        total_occupancy / lanes.len() as f64
    };

    // Compute intersection-level congestion (average occupancy of outgoing lanes).
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

    // Compute intersection waiting times.
    // Here we assume that each lane has a `waiting_time` field.
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
        average_lane_occupancy,
        intersection_congestion,
        intersection_waiting_time, // waiting time data added
    }
}

/// This structure packages both current and predicted traffic data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficUpdate {
    pub current_data: TrafficData,
    pub predicted_data: TrafficData,
    pub timestamp: u64,
}

/// Returns the current timestamp (in seconds since the UNIX epoch).
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Sends a traffic update to the Traffic Light Controller via a channel.
pub fn send_update_to_controller(update: TrafficUpdate, tx: &Sender<TrafficUpdate>) {
    if let Err(e) = tx.send(update) {
        log::info!("Failed to send traffic update to controller: {}", e);
    }
}

/// Analyze traffic data to detect congestion. Returns a list of alerts.
pub fn analyze_traffic(data: &TrafficData) -> Vec<CongestionAlert> {
    let mut alerts = Vec::new();

    if data.average_lane_occupancy > 0.75 {
        alerts.push(CongestionAlert {
            intersection: None,
            message: format!(
                "System-wide congestion is high (occupancy: {:.2})",
                data.average_lane_occupancy
            ),
            recommended_action: String::from("Reroute or adjust signals globally."),
        });
    }

    for (&int_id, &cong) in &data.intersection_congestion {
        if cong > 0.80 {
            alerts.push(CongestionAlert {
                intersection: Some(int_id),
                message: format!(
                    "Intersection {:?} is heavily congested ({:.2})",
                    int_id, cong
                ),
                recommended_action: String::from("Adjust light timings or partial rerouting."),
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
        average_lane_occupancy: data.average_lane_occupancy, // global occupancy remains unchanged
        intersection_congestion: new_congestion,
        intersection_waiting_time: new_waiting_time,
    }
}
/// Send alerts to the control system (here we just print them).
pub fn send_congestion_alerts(alerts: &[CongestionAlert]) {
    for alert in alerts {
        println!("--- Congestion Alert ---");
        if let Some(int_id) = alert.intersection {
            println!("Affected Intersection: {:?}", int_id);
        }
        println!("Message: {}", alert.message);
        println!("Recommended Action: {}", alert.recommended_action);
    }
}

/// Represents a suggested route update.
#[derive(Debug, Clone)]
pub struct RouteUpdate {
    pub new_route: Vec<Lane>,
    pub reason: String,
}

// If occupancy is high, attempt to generate a new route that avoids certain intersections.
pub fn generate_route_update(
    data: &TrafficData,
    current_route: &[Lane],
    avoid_intersections: &[IntersectionId],
    all_lanes: &[Lane],
) -> Option<RouteUpdate> {
    let occupancy_threshold = 0.75;
    if data.average_lane_occupancy > occupancy_threshold {
        println!(
            "High occupancy detected: {:.2}. Generating a less traffic route...",
            data.average_lane_occupancy
        );
        let current_intersection = current_route.first().map(|lane| lane.from)?;
        let target_intersection = current_route.last().map(|lane| lane.to)?;
        if avoid_intersections.contains(&target_intersection) {
            println!("Target intersection is in the avoid list. Skipping route update.");
            return None;
        }

        // Call your route generation algorithm (defined in route_generation.rs).
        if let Some(new_route) =
            crate::simulation_engine::route_generation::generate_shortest_lane_route(
                all_lanes,
                current_intersection,
                target_intersection,
            )
        {
            return Some(RouteUpdate {
                new_route,
                reason: format!(
                    "Average occupancy {:.2} exceeded threshold {:.2}; rerouting suggested.",
                    data.average_lane_occupancy, occupancy_threshold
                ),
            });
        }
    }
    None
}

/// Represents a traffic light adjustment recommendation.
#[derive(Debug, Clone)]
pub struct SignalAdjustment {
    pub intersection_id: IntersectionId,
    pub add_seconds_green: u32,
}

/// Generate recommended traffic light adjustments based on congestion.
pub fn generate_signal_adjustments(data: &TrafficData) -> Vec<SignalAdjustment> {
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

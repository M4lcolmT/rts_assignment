use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::simulation_engine::intersections::{Intersection, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::vehicles::Vehicle;

#[derive(Debug, Clone)]
pub struct AccidentEvent {
    pub lane: Lane,
    pub severity: u8,
}

/// When an accident occurs, vehicles already on the accident lane must wait,
/// with waiting time increased based on severity. Upcoming vehicles will avoid this lane
/// (via generate_route_update logic). Thus, this handler no longer re-routes vehicles.
pub fn handle_accident_event(
    accident: &AccidentEvent,
    vehicles: &mut Vec<(Vehicle, Vec<Lane>)>,
    _lanes: &[Lane],
) {
    println!(
        "[FlowAnalyzer] Accident reported on lane '{}' (severity: {})",
        accident.lane.name, accident.severity
    );

    // Iterate over vehicles to check if they are in the accident lane.
    for (vehicle, route) in vehicles.iter_mut() {
        // Check if the vehicle's current lane is the accident lane.
        if let Some(current_lane) = route.first() {
            if current_lane.name == accident.lane.name {
                // Instead of re-routing, vehicles already on the accident lane will wait.
                // The waiting time can be increased based on the severity of the accident.
                // (For example, severity 1-5 could add 2-10 seconds respectively.)
                let extra_wait_time = accident.severity as u32 * 2;
                println!(
                    "[FlowAnalyzer] Vehicle {} is on the accident lane. Increased waiting time by {} seconds.",
                    vehicle.id, extra_wait_time
                );
                // If the Vehicle struct had a waiting time field, update it accordingly.
                // TODO: vehicle.waiting_time += extra_wait_time;
            } else if route.iter().any(|lane| lane.to == accident.lane.to) {
                // For vehicles not yet on the accident lane, simply log that they will
                println!(
                    "[FlowAnalyzer] Vehicle {} is approaching the accident area. It will be re-routed by the update mechanism.",
                    vehicle.id
                );
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficData {
    pub total_vehicles: usize,
    pub lane_occupancy: HashMap<String, f64>,
    pub intersection_congestion: HashMap<IntersectionId, f64>,
    pub intersection_waiting_time: HashMap<IntersectionId, f64>,
}

#[derive(Debug, Clone)]
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
        intersection_congestion,
        intersection_waiting_time,
    }
}

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

    // Generate intersection-level alerts as before.
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

/// If one or more lanes in the current route are congested, attempt to generate a new route
/// that avoids those congested lanes and any lane with an accident.
/// The caller should update the vehicle's route with the returned RouteUpdate.
pub fn generate_route_update(
    data: &TrafficData,
    current_route: &[Lane],
    all_lanes: &[Lane],
    accident_lane: Option<&Lane>,
    vehicle_id: u64, // New parameter to specify which vehicle's route is being updated.
) -> Option<RouteUpdate> {
    let occupancy_threshold = 0.75;

    // Identify congested lanes in the current route.
    let congested_lanes: Vec<&Lane> = current_route
        .iter()
        .filter(|lane| {
            data.lane_occupancy
                .get(&lane.name)
                .map(|&occ| occ > occupancy_threshold)
                .unwrap_or(false)
        })
        .collect();

    // If no congested lanes are found, no route update is needed.
    if congested_lanes.is_empty() {
        return None;
    }

    // Determine the start and target intersections from the current route.
    let current_intersection = current_route.first().map(|lane| lane.from)?;
    let target_intersection = current_route.last().map(|lane| lane.to)?;

    println!(
        "Vehicle {}: Congested lanes detected in current route: {:?}",
        vehicle_id,
        congested_lanes
            .iter()
            .map(|lane| lane.name.as_str())
            .collect::<Vec<_>>()
    );

    // TEST: to check if it completely removes the congested lanes and not add them back
    // Filter out lanes that are congested and, additionally, filter out the lane with an accident.
    let filtered_lanes: Vec<Lane> = all_lanes
        .iter()
        .filter(|lane| {
            // Exclude lane if its occupancy is above threshold.
            let occupancy_ok = data
                .lane_occupancy
                .get(&lane.name)
                .map(|&occ| occ <= occupancy_threshold)
                .unwrap_or(true);
            // Exclude lane if it is the accident lane.
            let not_accident = if let Some(acc_lane) = accident_lane {
                lane.name != acc_lane.name
            } else {
                true
            };
            occupancy_ok && not_accident
        })
        .cloned()
        .collect();

    // Generate a new route using the filtered lanes.
    if let Some(new_route) =
        crate::simulation_engine::route_generation::generate_shortest_lane_route(
            &filtered_lanes,
            current_intersection,
            target_intersection,
        )
    {
        // Print out the new route with the vehicle ID for debugging/logging.
        println!(
            "Vehicle {}: New route generated: {:?}",
            vehicle_id,
            new_route
                .iter()
                .map(|lane| lane.name.as_str())
                .collect::<Vec<_>>()
        );
        return Some(RouteUpdate {
            new_route,
            reason: format!(
                "Congested lanes (occupancy > {:.2}) detected; rerouting suggested to avoid these segments.",
                occupancy_threshold
            ),
        });
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

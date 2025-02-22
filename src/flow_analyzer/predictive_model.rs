use std::collections::HashMap;

use crate::simulation_engine::intersections::{Intersection, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::vehicles::Vehicle;

/// Aggregated traffic data.
#[derive(Debug, Clone)]
pub struct TrafficData {
    pub total_vehicles: usize,
    pub average_lane_occupancy: f64,
    pub intersection_congestion: HashMap<IntersectionId, f64>,
}

/// A congestion alert with a message and recommended action.
#[derive(Debug, Clone)]
pub struct CongestionAlert {
    pub intersection: Option<IntersectionId>,
    pub message: String,
    pub recommended_action: String,
}

/// Collect real-time data from lanes, vehicles, and intersections.
pub fn collect_traffic_data(
    lanes: &[Lane],
    vehicles: &[Vehicle],
    intersections: &[Intersection],
) -> TrafficData {
    let total_vehicles = vehicles.len();

    // Average lane occupancy
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

    // Intersection-level congestion (simple average of outgoing lanes)
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

    TrafficData {
        total_vehicles,
        average_lane_occupancy,
        intersection_congestion,
    }
}

/// Analyze traffic data to find congestion. Returns a list of alerts.
pub fn analyze_traffic(data: &TrafficData) -> Vec<CongestionAlert> {
    let mut alerts = Vec::new();

    // Check overall average occupancy
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

    // Check each intersection's congestion
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

/// Predict future traffic conditions by multiplying occupancy by 1.1 (clamped at 1.0).
pub fn predict_future_traffic(data: &TrafficData) -> TrafficData {
    let factor = 1.1;
    let mut new_congestion = HashMap::new();
    for (&int_id, &val) in &data.intersection_congestion {
        new_congestion.insert(int_id, (val * factor).min(1.0));
    }

    TrafficData {
        total_vehicles: data.total_vehicles,
        average_lane_occupancy: (data.average_lane_occupancy * factor).min(1.0),
        intersection_congestion: new_congestion,
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

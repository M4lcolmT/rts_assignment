pub mod predictive_model;

// Re-export the items from predictive_model
pub use predictive_model::{
    analyze_traffic, collect_traffic_data, generate_route_update, predict_future_traffic,
    send_congestion_alerts, CongestionAlert, TrafficData,
};

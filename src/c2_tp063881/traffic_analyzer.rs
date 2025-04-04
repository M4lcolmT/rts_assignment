use crate::global_variables::{
    AMQP_URL, QUEUE_CONGESTION_ALERTS, QUEUE_TRAFFIC_DATA, QUEUE_TRAFFIC_EVENTS,
};
use crate::shared_data::{
    current_timestamp, AccidentInfo, CongestionAlert, TrafficData, TrafficEvent, TrafficUpdate,
};
use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result as AmiquipResult,
};
use serde_json;
use std::sync::{Arc, Mutex}; 
use std::collections::{HashMap, VecDeque};
use tokio::{self, task, time::Duration};

#[derive(Debug, Clone)]
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

// Analyze congestion from intersection data.
pub fn analyze_traffic_data(data: &TrafficData) -> Vec<CongestionAlert> {
    let ts = current_timestamp();
    let mut alerts = Vec::new();
    for (int_id, &cong) in &data.intersection_congestion {
        if cong > 0.50 {
            alerts.push(CongestionAlert {
                timestamp: ts,
                intersection: Some(int_id.clone()),
                message: format!("Intersection {} is heavily congested ({:.2})", int_id, cong),
                congestion_perc: cong,
                recommended_action: "Adjust traffic light timings to avoid congestion.".to_string(),
            });
        }
    }
    alerts
}

pub fn analyze_traffic_events(update: TrafficUpdate) -> TrafficEvent {
    let ts = current_timestamp();
    let mut total_delay = 0;
    let mut count_delay = 0;
    let mut accident_list = Vec::new();

    for v in &update.current_data.vehicle_data {
        total_delay += v.waiting_time;
        count_delay += 1;
        if let Some(ts) = v.accident_timestamp {
            accident_list.push(AccidentInfo {
                vehicle_id: v.id,
                accident_timestamp: ts,
                severity: v.severity,
                current_lane: v.current_lane.clone(),
            });
        }
    }
    let avg_delay = if count_delay > 0 {
        // round to 2 decimal places
        (total_delay as f64 / count_delay as f64 * 100.0).round() / 100.0
    } else {
        0.0
    };

    TrafficEvent {
        timestamp: ts,
        average_vehicle_delay: avg_delay,
        total_accidents: accident_list.len(),
        accident_details: accident_list,
    }
}

// Predict future traffic based on weighted historical data.
pub fn predict_future_traffic_weighted(
    data: &TrafficData,
    historical: &HistoricalData,
    alpha: f64,
) -> TrafficData {
    let mut new_congestion = HashMap::new();
    let mut new_waiting_time = HashMap::new();

    for (int_id, &current_occ) in &data.intersection_congestion {
        let hist_occ = historical.average_occupancy_for(int_id);
        let predicted_occ = alpha * current_occ + (1.0 - alpha) * hist_occ;
        new_congestion.insert(int_id.clone(), predicted_occ.min(1.0));
    }

    for (int_id, &current_wait) in &data.intersection_waiting_time {
        let hist_wait = historical.average_waiting_time_for(int_id);
        let predicted_wait = alpha * current_wait + (1.0 - alpha) * hist_wait;
        new_waiting_time.insert(int_id.clone(), predicted_wait);
    }

    TrafficData {
        lane_occupancy: data.lane_occupancy.clone(),
        accident_lanes: data.accident_lanes.clone(),
        intersection_congestion: new_congestion,
        intersection_waiting_time: new_waiting_time,
        vehicle_data: data.vehicle_data.clone(),
    }
}

pub async fn start_analyzer_rabbitmq() -> AmiquipResult<()> {
    task::spawn_blocking(|| -> AmiquipResult<()> {
        let historical = Arc::new(Mutex::new(HistoricalData::new(10)));
        let latest_data = Arc::new(Mutex::new(None::<TrafficData>));

        // Spawn a separate thread that prints future traffic predictions every 5 seconds
        {
            let historical = Arc::clone(&historical);
            let latest_data = Arc::clone(&latest_data);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(10)).await;
                    // Lock the historical data to check if we have sufficient samples.
                    let hist = historical.lock().unwrap();
                    let sufficient = hist.occupancy_history.values().any(|deque| deque.len() >= hist.capacity);
            
                    if sufficient {
                        if let Some(ref current_data) = *latest_data.lock().unwrap() {
                            // Use alpha = 0.7 for the weighted prediction
                            let predicted = predict_future_traffic_weighted(current_data, &hist, 0.7);
                            println!(
                                "Future Traffic Prediction: Congestion: {:?}, Waiting Time: {:?}",
                                predicted.intersection_congestion,
                                predicted.intersection_waiting_time
                            );
                        }
                    }
                }
            });
            
        }

        let mut connection = Connection::insecure_open(AMQP_URL)?;
        let channel = connection.open_channel(None)?;
        let exchange = Exchange::direct(&channel);

        let traffic_data_queue = channel.queue_declare(QUEUE_TRAFFIC_DATA, QueueDeclareOptions::default())?;
        let consumer = traffic_data_queue.consume(ConsumerOptions::default())?;

        println!("[Analyzer] Waiting for TrafficUpdate on 'traffic_data'...");

        channel.queue_declare(QUEUE_CONGESTION_ALERTS, QueueDeclareOptions::default())?;
        channel.queue_declare(QUEUE_TRAFFIC_EVENTS, QueueDeclareOptions::default())?;

        for message in consumer.receiver() {
            println!("Received message from simulation to flow analyzer");

            match message {
                ConsumerMessage::Delivery(delivery) => {
                    if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                        if let Ok(update) = serde_json::from_str::<TrafficUpdate>(json_str) {
                            {
                                let mut hist = historical.lock().unwrap();
                                hist.update_occupancy(&update.current_data);
                                hist.update_waiting_time(&update.current_data.intersection_waiting_time);
                            }

                            {
                                let mut ld = latest_data.lock().unwrap();
                                *ld = Some(update.current_data.clone());
                            }

                            let alerts = analyze_traffic_data(&update.current_data);
                            if !alerts.is_empty() {
                                for alert in &alerts {
                                    if let Ok(alert_json) = serde_json::to_string(alert) {
                                        exchange.publish(Publish::new(
                                            alert_json.as_bytes(),
                                            QUEUE_CONGESTION_ALERTS,
                                        ))?;
                                    }
                                }
                                println!(
                                    "[Analyzer] Published {} congestion alerts to 'congestion_alerts'",
                                    alerts.len()
                                );
                            }

                            let traffic_event = analyze_traffic_events(update);
                            if let Ok(event_json) = serde_json::to_string(&traffic_event) {
                                exchange.publish(Publish::new(
                                    event_json.as_bytes(),
                                    QUEUE_TRAFFIC_EVENTS,
                                ))?;
                                println!(
                                    "[Analyzer] Published TrafficEvent to 'traffic_events': {:?}",
                                    traffic_event
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
    })
    .await
    .unwrap()
}

use crate::c1_tp063879::intersections::{Intersection, IntersectionControl, IntersectionId};
use crate::c1_tp063879::lanes::Lane;
use crate::global_variables::{AMQP_URL, QUEUE_CONGESTION_ALERTS, QUEUE_LIGHT_ADJUSTMENTS};
use crate::shared_data::{current_timestamp, CongestionAlert, LightAdjustment};
use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result as AmiquipResult,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task;
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub struct TrafficLightPhase {
    pub green_lanes: Vec<String>,
    pub duration: u64, // Duration in seconds
}

pub struct IntersectionController {
    pub intersection: Intersection,
    pub phases: Vec<TrafficLightPhase>,
    pub current_phase_index: usize,
    pub elapsed_in_phase: u64,
    pub all_lanes: Vec<String>,
    pub emergency_override: Option<Vec<String>>,
}

impl IntersectionController {
    pub fn new(
        intersection: Intersection,
        phases: Vec<TrafficLightPhase>,
        all_lanes: Vec<String>,
    ) -> Self {
        Self {
            intersection,
            phases,
            current_phase_index: 0,
            elapsed_in_phase: 0,
            all_lanes,
            emergency_override: None,
        }
    }

    // Increases the elapsed time and cycles the phase if the current phase's duration is reached.
    pub fn update(&mut self) {
        if self.emergency_override.is_some() {
            // Do not cycle phases during emergency override.
            return;
        }
        self.elapsed_in_phase += 1;
        let current_phase = &self.phases[self.current_phase_index];
        if self.elapsed_in_phase >= current_phase.duration {
            self.elapsed_in_phase = 0;
            self.current_phase_index = (self.current_phase_index + 1) % self.phases.len();
            self.apply_current_phase();
        }
    }

    // Prints the currently active green and red lanes.
    pub fn apply_current_phase(&self) {
        if let Some(ref override_lanes) = self.emergency_override {
            let red_lanes: Vec<String> = self
                .all_lanes
                .iter()
                .filter(|lane| !override_lanes.contains(lane))
                .cloned()
                .collect();
            println!(
                "Intersection {:?} EMERGENCY OVERRIDE: Green for lanes: {:?} and Red for lanes: {:?}",
                self.intersection.id, override_lanes, red_lanes
            );
        } else {
            let current_green = &self.phases[self.current_phase_index].green_lanes;
            let red_lanes: Vec<String> = self
                .all_lanes
                .iter()
                .filter(|lane| !current_green.contains(lane))
                .cloned()
                .collect();
            println!(
                "Intersection {:?} switching to phase {}: Green for lanes: {:?} and Red for lanes: {:?}",
                self.intersection.id,
                self.current_phase_index,
                current_green,
                red_lanes
            );
        }
    }

    // Adjust phase durations based on predicted congestion values.

    pub fn adjust_phase_durations_based_on_prediction(
        &mut self,
        predicted_data: &HashMap<String, f64>,
    ) {
        for phase in self.phases.iter_mut() {
            let mut extra_seconds = 0;
            for lane in &phase.green_lanes {
                if let Some(&congestion) = predicted_data.get(lane) {
                    if congestion > 0.8 {
                        extra_seconds = extra_seconds.max(1); // add extra 1 seconds if congestion is high
                    }
                }
            }
            let new_duration = 1 + extra_seconds; // base duration is 1 seconds
            if phase.duration != new_duration {
                println!(
                    "Adjusting phase duration for lanes {:?} from {} to {} seconds based on predicted traffic",
                    phase.green_lanes, phase.duration, new_duration
                );
                phase.duration = new_duration;
            }
        }
    }

    // Admin function to directly set a phase's duration.
    pub fn set_phase_duration(&mut self, phase_index: usize, new_duration: u64) {
        if phase_index < self.phases.len() {
            println!(
                "Admin adjusted phase {} duration at intersection {:?} from {} to {} seconds",
                phase_index, self.intersection.id, self.phases[phase_index].duration, new_duration
            );
            self.phases[phase_index].duration = new_duration;
        } else {
            println!(
                "Invalid phase index {} for intersection {:?}",
                phase_index, self.intersection.id
            );
        }
    }

    // Sets an emergency override for the intersection.
    pub fn set_emergency_override(&mut self, emergency_route: Vec<String>) {
        self.emergency_override = Some(emergency_route);
        self.apply_current_phase();
    }

    // Clears the emergency override.
    pub fn clear_emergency_override(&mut self) {
        if self.emergency_override.is_some() {
            println!(
                "Clearing emergency override for intersection {:?}",
                self.intersection.id
            );
        }
        self.emergency_override = None;
        self.apply_current_phase();
    }
}

pub struct TrafficLightController {
    pub controllers: HashMap<IntersectionId, IntersectionController>,
}

impl TrafficLightController {
    // Creates a controller for each intersection with traffic light control.
    // Lanes are grouped into phases based on their orientation.
    pub fn initialize(intersections: Vec<Intersection>, lanes: &[Lane]) -> Self {
        let mut controllers = HashMap::new();

        for intersection in intersections {
            if intersection.control == IntersectionControl::TrafficLight {
                let connected_lanes: Vec<&Lane> = lanes
                    .iter()
                    .filter(|lane| lane.from == intersection.id)
                    .collect();
                if connected_lanes.is_empty() {
                    continue;
                }
                // Group lanes by orientation.
                let horizontal: Vec<String> = connected_lanes
                    .iter()
                    .filter(|lane| lane.from.0 == lane.to.0)
                    .map(|lane| lane.name.clone())
                    .collect();
                let vertical: Vec<String> = connected_lanes
                    .iter()
                    .filter(|lane| lane.from.1 == lane.to.1)
                    .map(|lane| lane.name.clone())
                    .collect();

                let mut phases = Vec::new();
                if !horizontal.is_empty() && !vertical.is_empty() {
                    // Two phases: one for horizontal lanes and one for vertical lanes.
                    phases.push(TrafficLightPhase {
                        green_lanes: horizontal.clone(),
                        duration: 8, // most of the vehicles take around 1-8 seconds to travel from one intersection to another. vehicles with longer travel time will have to wait in the queue
                    });
                    phases.push(TrafficLightPhase {
                        green_lanes: vertical.clone(),
                        duration: 8,
                    });
                } else {
                    // Single phase with all connected lanes.
                    let all: Vec<String> = connected_lanes
                        .iter()
                        .map(|lane| lane.name.clone())
                        .collect();
                    phases.push(TrafficLightPhase {
                        green_lanes: all,
                        duration: 8,
                    });
                }

                // All lane names for display purposes.
                let all_lane_names: Vec<String> = connected_lanes
                    .iter()
                    .map(|lane| lane.name.clone())
                    .collect();
                let controller =
                    IntersectionController::new(intersection.clone(), phases, all_lane_names);
                controllers.insert(intersection.id, controller);
            }
        }
        Self { controllers }
    }

    // Calls update() on all individual intersection controllers.
    pub fn update_all(&mut self) {
        for controller in self.controllers.values_mut() {
            controller.update();
        }
    }

    // Checks if a given lane at an intersection is currently green.
    pub fn is_lane_green(&self, intersection_id: IntersectionId, lane_name: &str) -> bool {
        if let Some(ctrl) = self.controllers.get(&intersection_id) {
            if let Some(ref override_lanes) = ctrl.emergency_override {
                return override_lanes.contains(&lane_name.to_string());
            }
            let current_phase = &ctrl.phases[ctrl.current_phase_index];
            return current_phase.green_lanes.contains(&lane_name.to_string());
        }
        // If intersection is not controlled by a traffic light, default to green.
        true
    }

    // Sets an emergency override for a given intersection.
    pub fn set_emergency_override_route(
        &mut self,
        intersection_id: IntersectionId,
        emergency_route: Vec<String>,
    ) {
        if let Some(ctrl) = self.controllers.get_mut(&intersection_id) {
            ctrl.set_emergency_override(emergency_route);
        }
    }

    // Clears an emergency override for a given intersection.
    pub fn clear_emergency_override(&mut self, intersection_id: IntersectionId) {
        if let Some(ctrl) = self.controllers.get_mut(&intersection_id) {
            ctrl.clear_emergency_override();
        }
    }

    // Admin function to adjust a specific phase's duration.
    pub fn set_phase_duration(
        &mut self,
        intersection_id: IntersectionId,
        phase_index: usize,
        new_duration: u64,
    ) {
        if let Some(ctrl) = self.controllers.get_mut(&intersection_id) {
            ctrl.set_phase_duration(phase_index, new_duration);
        }
    }

    // Adjusts phase durations for an intersection based on predicted traffic data.
    pub fn adjust_phases_based_on_prediction(
        &mut self,
        intersection_id: IntersectionId,
        predicted_data: &HashMap<String, f64>,
    ) {
        if let Some(ctrl) = self.controllers.get_mut(&intersection_id) {
            ctrl.adjust_phase_durations_based_on_prediction(predicted_data);
        }
    }

    // Runs a dedicated update loop that periodically updates all traffic lights.
    // This function is intended to be spawned as an async task.
    pub async fn run_update_loop(controller: Arc<Mutex<Self>>) {
        loop {
            {
                let mut ctrl = controller.lock().unwrap();
                ctrl.update_all();
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
}

pub async fn start_traffic_controller_rabbitmq() -> AmiquipResult<()> {
    task::spawn_blocking(|| -> AmiquipResult<()> {
        let mut connection = Connection::insecure_open(AMQP_URL)?;
        let channel = connection.open_channel(None)?;
        let exchange = Exchange::direct(&channel);
        let congestion_alert_queue =
            channel.queue_declare(QUEUE_CONGESTION_ALERTS, QueueDeclareOptions::default())?;
        let consumer = congestion_alert_queue.consume(ConsumerOptions::default())?;
        println!("[TrafficController] Waiting for congestion alerts on 'congestion_alerts'...");

        channel.queue_declare(QUEUE_LIGHT_ADJUSTMENTS, QueueDeclareOptions::default())?;

        for message in consumer.receiver() {
            println!("Received message in TrafficController");
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let ts = current_timestamp();
                    if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                        if let Ok(alert) = serde_json::from_str::<CongestionAlert>(json_str) {
                            println!("[TrafficController] Got CongestionAlert: {:?}", alert);
                            if let Some(int_id) = alert.intersection {
                                // TODO: Temporarily, for demonstration, publish a fixed additional duration adjustment.
                                let adjustment = LightAdjustment {
                                    timestamp: ts,
                                    intersection_id: int_id.to_string(),
                                    add_seconds_green: 5,
                                };
                                if let Ok(adj_json) = serde_json::to_string(&adjustment) {
                                    exchange.publish(Publish::new(
                                        adj_json.as_bytes(),
                                        QUEUE_LIGHT_ADJUSTMENTS,
                                    ))?;
                                    println!(
                                        "[TrafficController] Published LightAdjustment: {:?}",
                                        adjustment
                                    );
                                }
                            }
                        }
                    }
                    consumer.ack(delivery)?;
                }
                other => {
                    println!("[TrafficController] Consumer ended: {:?}", other);
                    break;
                }
            }
        }
        connection.close()
    })
    .await
    .unwrap()
}

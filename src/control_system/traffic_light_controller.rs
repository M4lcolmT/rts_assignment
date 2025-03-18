// traffic_controller_system.rs

use crate::flow_analyzer::traffic_analyzer::CongestionAlert;
use crate::simulation_engine::intersections::{Intersection, IntersectionControl, IntersectionId};
use crate::simulation_engine::lanes::Lane;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// We keep your original structures:
#[derive(Debug, Clone)]
pub struct TrafficLightPhase {
    pub green_lanes: Vec<String>,
    pub duration: u64,
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

    pub fn update(&mut self) {
        if self.emergency_override.is_some() {
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

    pub fn apply_current_phase(&self) {
        if let Some(ref override_lanes) = self.emergency_override {
            let red_lanes: Vec<String> = self
                .all_lanes
                .iter()
                .filter(|lane| !override_lanes.contains(lane))
                .cloned()
                .collect();
            println!(
                "Intersection {:?} emergency override: Green for lanes: {:?} and Red for lanes: {:?}",
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

    pub fn set_emergency_override(&mut self, lanes_to_green: Vec<String>) {
        self.emergency_override = Some(lanes_to_green);
        self.apply_current_phase();
    }

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
    pub fn initialize(intersections: Vec<Intersection>, lanes: &[Lane]) -> Self {
        let mut controllers = HashMap::new();

        for intersection in intersections {
            if intersection.control == IntersectionControl::TrafficLight {
                let connected_lanes: Vec<String> = lanes
                    .iter()
                    .filter(|lane| lane.from == intersection.id)
                    .map(|lane| lane.name.clone())
                    .collect();

                if connected_lanes.is_empty() {
                    continue;
                }

                let phases: Vec<TrafficLightPhase> = connected_lanes
                    .iter()
                    .map(|ln| TrafficLightPhase {
                        green_lanes: vec![ln.clone()],
                        duration: 5,
                    })
                    .collect();

                let controller = IntersectionController::new(
                    intersection.clone(),
                    phases,
                    connected_lanes.clone(),
                );
                controllers.insert(intersection.id, controller);
            }
        }

        Self { controllers }
    }

    pub fn update_all(&mut self) {
        for controller in self.controllers.values_mut() {
            controller.update();
        }
    }

    pub fn is_lane_green(&self, intersection_id: IntersectionId, lane_name: &str) -> bool {
        if let Some(ctrl) = self.controllers.get(&intersection_id) {
            if let Some(ref override_lanes) = ctrl.emergency_override {
                return override_lanes.contains(&lane_name.to_string());
            }
            let current_phase = &ctrl.phases[ctrl.current_phase_index];
            current_phase.green_lanes.contains(&lane_name.to_string())
        } else {
            true
        }
    }

    pub fn set_emergency_override(
        &mut self,
        intersection_id: IntersectionId,
        emergency_lane: &str,
        all_lanes: &[Lane],
    ) {
        if let Some(ctrl) = self.controllers.get_mut(&intersection_id) {
            let mut green_lanes = vec![emergency_lane.to_string()];
            if let Some(em_lane_obj) = all_lanes.iter().find(|l| l.name == emergency_lane) {
                let opposite_name = format!(
                    "({},{}) -> ({},{})",
                    em_lane_obj.to.0, em_lane_obj.to.1, em_lane_obj.from.0, em_lane_obj.from.1
                );
                if ctrl.all_lanes.contains(&opposite_name) {
                    green_lanes.push(opposite_name);
                }
            }
            ctrl.set_emergency_override(green_lanes);
        }
    }

    pub fn clear_emergency_override(&mut self, intersection_id: IntersectionId) {
        if let Some(ctrl) = self.controllers.get_mut(&intersection_id) {
            ctrl.clear_emergency_override();
        }
    }
}

// === AMIQIP ===
// Listens for "congestion_alerts" and publishes "light_adjustments".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightAdjustmentMsg {
    pub intersection_id: String,
    pub add_seconds_green: u32,
}

use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result as AmiquipResult,
};

pub fn start_traffic_controller_rabbitmq() -> AmiquipResult<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
    let channel = connection.open_channel(None)?;

    let exchange = Exchange::direct(&channel);

    let congestion_alert_queue =
        channel.queue_declare("congestion_alerts", QueueDeclareOptions::default())?;
    let consumer = congestion_alert_queue.consume(ConsumerOptions::default())?;
    println!("[TrafficController] Waiting for congestion alerts on 'congestion_alerts'...");

    channel.queue_declare("light_adjustments", QueueDeclareOptions::default())?;

    // 5) Start consuming CongestionAlert messages
    for message in consumer.receiver() {
        println!("received message from simulation to light controller");
        match message {
            ConsumerMessage::Delivery(delivery) => {
                if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                    if let Ok(alert) = serde_json::from_str::<CongestionAlert>(json_str) {
                        println!("[TrafficController] Got CongestionAlert: {:?}", alert);

                        if let Some(int_id) = alert.intersection {
                            let adjustment = LightAdjustmentMsg {
                                intersection_id: int_id.to_string(),
                                add_seconds_green: 10,
                            };

                            if let Ok(adj_json) = serde_json::to_string(&adjustment) {
                                exchange.publish(Publish::new(
                                    adj_json.as_bytes(),
                                    "light_adjustments",
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
}

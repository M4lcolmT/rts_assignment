use rts_assignment::control_system::traffic_light_controller::start_traffic_controller_rabbitmq;

fn main() {
    env_logger::init();
    println!("Starting traffic controller...");
    if let Err(e) = start_traffic_controller_rabbitmq() {
        eprintln!("Controller error: {}", e);
    }
}

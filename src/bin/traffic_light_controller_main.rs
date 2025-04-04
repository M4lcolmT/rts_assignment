use rts_assignment::c3_tp063987::traffic_light_controller::start_traffic_controller_rabbitmq;

#[tokio::main]
async fn main() {
    env_logger::init();
    println!("Starting traffic controller...");

    if let Err(e) = start_traffic_controller_rabbitmq().await {
        eprintln!("Controller error: {}", e);
    }
}

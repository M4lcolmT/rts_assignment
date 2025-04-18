use rts_assignment::c2_tp063881::traffic_analyzer::start_analyzer_rabbitmq;

#[tokio::main]
async fn main() {
    env_logger::init();
    println!("Starting traffic analyzer...");

    if let Err(e) = start_analyzer_rabbitmq().await {
        eprintln!("Analyzer error: {}", e);
    }
}

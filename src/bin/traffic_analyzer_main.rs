use rts_assignment::flow_analyzer::traffic_analyzer::start_analyzer_rabbitmq;

fn main() {
    env_logger::init();
    println!("Starting traffic analyzer...");
    if let Err(e) = start_analyzer_rabbitmq() {
        eprintln!("Analyzer error: {}", e);
    }
}

use rts_assignment::c4_tp071994::traffic_monitoring_system::{
    listen_congestion_alerts, listen_light_adjustments, listen_traffic_data, listen_traffic_event,
    run_cli,
};
use tokio::join;

#[tokio::main]
async fn main() {
    // Spawn listeners for the three RabbitMQ channels concurrently.
    let congestion_listener = tokio::spawn(async {
        if let Err(e) = listen_congestion_alerts().await {
            eprintln!("Error in congestion alerts listener: {}", e);
        }
    });
    let light_adjustments_listener = tokio::spawn(async {
        if let Err(e) = listen_light_adjustments().await {
            eprintln!("Error in light adjustments listener: {}", e);
        }
    });
    let traffic_data_listener = tokio::spawn(async {
        if let Err(e) = listen_traffic_data().await {
            eprintln!("Error in traffic data listener: {}", e);
        }
    });
    let traffic_event_listener = tokio::spawn(async {
        if let Err(e) = listen_traffic_event().await {
            eprintln!("Error in traffic event listener: {}", e);
        }
    });

    // Run the admin CLI concurrently.
    let cli_handle = tokio::spawn(async {
        run_cli().await;
    });

    // Wait for all tasks to complete (the CLI will exit on its own).
    let _ = join!(
        congestion_listener,
        light_adjustments_listener,
        traffic_data_listener,
        traffic_event_listener,
        cli_handle
    );
}

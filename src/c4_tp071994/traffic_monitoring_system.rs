use crate::global_variables::{
    AMQP_URL, QUEUE_CONGESTION_ALERTS, QUEUE_LIGHT_ADJUSTMENTS, QUEUE_TRAFFIC_DATA,
    QUEUE_TRAFFIC_EVENTS,
};
use crate::shared_data::{
    current_timestamp, AccidentInfo, CongestionAlert, LightAdjustment, TrafficEvent,
};
use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result as AmiquipResult,
};
use plotters::prelude::*;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{stdin, stdout, Write};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficDataRecord {
    pub timestamp: u64,
    pub raw_data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrafficEventSummary {
    pub timestamp: u64,
    pub average_vehicle_delay: f64,
    pub total_accidents: usize,
}

// Listens to the "congestion_alerts" queue and logs each incoming record.
pub async fn listen_congestion_alerts() -> AmiquipResult<()> {
    tokio::task::spawn_blocking(|| -> AmiquipResult<()> {
        let mut connection = Connection::insecure_open(AMQP_URL)?;
        let channel = connection.open_channel(None)?;
        let _exchange = Exchange::direct(&channel);
        let queue =
            channel.queue_declare(QUEUE_CONGESTION_ALERTS, QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        // println!("Listening for congestion alerts...");
        for message in consumer.receiver() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let ts = current_timestamp();
                    if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                        let record: CongestionAlert =
                            serde_json::from_str(json_str).unwrap_or(CongestionAlert {
                                timestamp: ts,
                                intersection: None,
                                message: json_str.to_string(),
                                congestion_perc: 0.0,
                                recommended_action: "No recomendations.".to_string(),
                            });
                        log_congestion_alert(record);
                    }
                    consumer.ack(delivery)?;
                }
                other => {
                    println!("Congestion alerts consumer ended: {:?}", other);
                    break;
                }
            }
        }
        connection.close()
    })
    .await
    .unwrap()
}

// Listens to the "light_adjustments" queue and logs each incoming record.
pub async fn listen_light_adjustments() -> AmiquipResult<()> {
    tokio::task::spawn_blocking(|| -> AmiquipResult<()> {
        let mut connection = Connection::insecure_open(AMQP_URL)?;
        let channel = connection.open_channel(None)?;
        let _exchange = Exchange::direct(&channel);
        let queue =
            channel.queue_declare(QUEUE_LIGHT_ADJUSTMENTS, QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        // println!("Listening for light adjustments...");
        for message in consumer.receiver() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let ts = current_timestamp();
                    if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                        let record: LightAdjustment =
                            serde_json::from_str(json_str).unwrap_or(LightAdjustment {
                                timestamp: ts,
                                intersection_id: "unknown".to_string(),
                                add_seconds_green: 0,
                            });
                        log_light_adjustment(record);
                    }
                    consumer.ack(delivery)?;
                }
                other => {
                    println!("Light adjustments consumer ended: {:?}", other);
                    break;
                }
            }
        }
        connection.close()
    })
    .await
    .unwrap()
}

// Listens to the "traffic_data" queue and logs each incoming record.
pub async fn listen_traffic_data() -> AmiquipResult<()> {
    tokio::task::spawn_blocking(|| -> AmiquipResult<()> {
        let mut connection = Connection::insecure_open(AMQP_URL)?;
        let channel = connection.open_channel(None)?;
        let _exchange = Exchange::direct(&channel);
        let queue = channel.queue_declare(QUEUE_TRAFFIC_DATA, QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        // println!("Listening for traffic data...");
        for message in consumer.receiver() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let ts = current_timestamp();
                    if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                        let record = TrafficDataRecord {
                            timestamp: ts,
                            raw_data: json_str.to_string(),
                        };
                        log_traffic_data(record);
                    }
                    consumer.ack(delivery)?;
                }
                other => {
                    println!("Traffic data consumer ended: {:?}", other);
                    break;
                }
            }
        }
        connection.close()
    })
    .await
    .unwrap()
}

pub async fn listen_traffic_event() -> AmiquipResult<()> {
    tokio::task::spawn_blocking(|| -> AmiquipResult<()> {
        let mut connection = Connection::insecure_open(AMQP_URL)?;
        let channel = connection.open_channel(None)?;
        let _exchange = Exchange::direct(&channel);
        let queue = channel.queue_declare(QUEUE_TRAFFIC_EVENTS, QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        // println!("Listening for traffic event...");
        for message in consumer.receiver() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let ts = current_timestamp();
                    if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                        // Use unwrap_or to fall back to a default TrafficEvent.
                        let record: TrafficEvent =
                            serde_json::from_str(json_str).unwrap_or(TrafficEvent {
                                timestamp: ts,
                                average_vehicle_delay: 0.0,
                                total_accidents: 0,
                                accident_details: Vec::new(),
                            });
                        log_traffic_event(record);
                    }
                    consumer.ack(delivery)?;
                }
                other => {
                    println!("Traffic event consumer ended: {:?}", other);
                    break;
                }
            }
        }
        connection.close()
    })
    .await
    .unwrap()
}

// Generic helper to log a record to a CSV file.
fn log_to_csv<T: Serialize>(filename: &str, record: &T) -> Result<(), Box<dyn Error>> {
    let file_exists = Path::new(filename).exists();
    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)?;
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(!file_exists)
        .from_writer(file);
    wtr.serialize(record)?;
    wtr.flush()?;
    Ok(())
}

// Logging functions for each type of record.
pub fn log_congestion_alert(record: CongestionAlert) {
    if let Err(e) = log_to_csv("congestion_alerts.csv", &record) {
        eprintln!("Error logging congestion alert: {}", e);
    }
}

pub fn log_light_adjustment(record: LightAdjustment) {
    if let Err(e) = log_to_csv("light_adjustments.csv", &record) {
        eprintln!("Error logging light adjustment: {}", e);
    }
}

pub fn log_traffic_data(record: TrafficDataRecord) {
    if let Err(e) = log_to_csv("traffic_data.csv", &record) {
        eprintln!("Error logging traffic data: {}", e);
    }
}

// Log the overall TrafficEvent and process its accident_details vector separately.
pub fn log_traffic_event(record: TrafficEvent) {
    // Create a summary record that omits the accident_details vector.
    let summary = TrafficEventSummary {
        timestamp: record.timestamp,
        average_vehicle_delay: record.average_vehicle_delay,
        total_accidents: record.total_accidents,
    };

    if let Err(e) = log_to_csv("traffic_event.csv", &summary) {
        eprintln!("Error logging traffic event summary: {}", e);
    }

    // Process each AccidentInfo record in the accident_details vector.
    for accident in record.accident_details {
        if let Err(e) = log_to_csv("accident_info.csv", &accident) {
            eprintln!("Error logging accident info: {}", e);
        }
    }
}

// Helper: Count records in a CSV file.
fn count_csv_records(filename: &str) -> Result<usize, Box<dyn Error>> {
    let file = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(file);
    let count = rdr.deserialize::<serde_json::Value>().count();
    Ok(count)
}

// Reads and displays records from "congestion_alerts.csv".
pub fn show_congestion_alerts() -> Result<(), Box<dyn Error>> {
    let file = File::open("congestion_alerts.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    println!("Congestion Alerts:");
    for result in rdr.deserialize() {
        let record: CongestionAlert = result?;
        println!("{:?}", record);
    }
    Ok(())
}

// Reads and displays records from "light_adjustments.csv".
pub fn show_light_adjustments() -> Result<(), Box<dyn Error>> {
    let file = File::open("light_adjustments.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    println!("Light Adjustments:");
    for result in rdr.deserialize() {
        let record: LightAdjustment = result?;
        println!("{:?}", record);
    }
    Ok(())
}

// Reads and displays records from "traffic_data.csv".
pub fn show_traffic_data() -> Result<(), Box<dyn Error>> {
    let file = File::open("traffic_data.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    println!("Traffic Data:");
    for result in rdr.deserialize() {
        let record: TrafficDataRecord = result?;
        println!("{:?}", record);
    }
    Ok(())
}

// Option 1: Display report summary with data counts.
pub fn generate_report_summary() -> Result<(), Box<dyn Error>> {
    println!("Generating Report Summary...");
    let congestion_count = count_csv_records("congestion_alerts.csv")?;
    let light_adjustments_count = count_csv_records("light_adjustments.csv")?;
    let traffic_data_count = count_csv_records("traffic_data.csv")?;
    let traffic_event_count = count_csv_records("traffic_event.csv")?;
    println!("Report Summary:");
    println!("Congestion Alerts: {} records", congestion_count);
    println!("Light Adjustments: {} records", light_adjustments_count);
    println!("Traffic Data: {} records", traffic_data_count);
    println!("Traffic Events: {} records", traffic_event_count);
    Ok(())
}

// Option 2: Show congestion report heatmap using Plotters.
pub fn show_congestion_heatmap() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("congestion_alerts.csv")?;
    let mut congestion_map: HashMap<(i32, i32), Vec<f64>> = HashMap::new();

    for result in rdr.deserialize() {
        let record: CongestionAlert = result?;
        if let Some(inter_str) = record.intersection {
            if let Some((x, y)) = parse_intersection(&inter_str) {
                let perc = record.congestion_perc;
                congestion_map.entry((x, y)).or_default().push(perc);
            }
        }
    }

    let mut avg_congestion: HashMap<(i32, i32), f64> = HashMap::new();
    for ((x, y), values) in congestion_map {
        let avg = values.iter().sum::<f64>() / values.len() as f64;
        avg_congestion.insert((x, y), avg);
    }

    let (grid_rows, grid_cols) = (4, 4);
    let (cell_width, cell_height) = (100, 100);
    let (image_width, image_height) = (grid_cols * cell_width, grid_rows * cell_height);

    let backend = BitMapBackend::new(
        "congestion_heatmap.png",
        (image_width as u32, image_height as u32),
    );
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;

    for row in 0..grid_rows {
        for col in 0..grid_cols {
            let congestion = avg_congestion
                .get(&(col as i32, row as i32))
                .cloned()
                .unwrap_or(0.0);
            let green_blue = (127.0 * (1.0 - congestion)).round() as u8;
            let fill_color = RGBColor(255, green_blue, green_blue);

            let x0 = col * cell_width;
            let y0 = row * cell_height;
            root.draw(&Rectangle::new(
                [(x0, y0), (x0 + cell_width, y0 + cell_height)],
                fill_color.filled(),
            ))?;

            root.draw(&Rectangle::new(
                [(x0, y0), (x0 + cell_width, y0 + cell_height)],
                &BLACK,
            ))?;

            let text = format!("({},{})\n{:.2}", col, row, congestion);
            let (text_x, text_y) = (x0 + cell_width / 2, y0 + cell_height / 2);
            root.draw(&Text::new(
                text,
                (text_x, text_y),
                TextStyle::from(("sans-serif", 15).into_font())
                    .color(&WHITE)
                    .pos(Pos::new(HPos::Center, VPos::Center)),
            ))?;
        }
    }

    root.present()?;
    println!("Congestion heatmap saved to congestion_heatmap.png");
    Ok(())
}

// Helper to parse an intersection string of the form "IntersectionId(x, y)".
fn parse_intersection(s: &str) -> Option<(i32, i32)> {
    let s = s.trim();
    if s.starts_with("IntersectionId(") && s.ends_with(")") {
        let inner = &s["IntersectionId(".len()..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 2 {
            if let (Ok(x), Ok(y)) = (
                parts[0].trim().parse::<i32>(),
                parts[1].trim().parse::<i32>(),
            ) {
                return Some((x, y));
            }
        }
    }
    None
}

// Option 3: Show traffic events data (average waiting time)
pub fn show_traffic_events() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path("traffic_event.csv")?;
    let events: Vec<TrafficEventSummary> = rdr.deserialize().filter_map(Result::ok).collect();

    if events.is_empty() {
        println!("No traffic event data available.");
        return Ok(());
    }

    let min_ts = events.iter().map(|e| e.timestamp).min().unwrap();
    let max_ts = events.iter().map(|e| e.timestamp).max().unwrap();
    let min_delay = events
        .iter()
        .map(|e| e.average_vehicle_delay)
        .fold(f64::INFINITY, f64::min);
    let max_delay = events
        .iter()
        .map(|e| e.average_vehicle_delay)
        .fold(f64::NEG_INFINITY, f64::max);

    let backend = BitMapBackend::new("traffic_events_scatterplot.png", (800, 600));
    let root = backend.into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Traffic Event Average Vehicle Delay", ("sans-serif", 20))
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(min_ts..max_ts, min_delay..max_delay)?;

    chart.configure_mesh().draw()?;
    chart.draw_series(
        events
            .iter()
            .map(|e| Circle::new((e.timestamp, e.average_vehicle_delay), 5, RED.filled())),
    )?;

    root.present()?;
    println!("Traffic events scatterplot saved to traffic_events_scatterplot.png");

    let accident_file = "accident_info.csv";
    if Path::new(accident_file).exists() {
        let mut rdr = csv::Reader::from_path(accident_file)?;
        let mut accident_count = 0;
        println!("Accident details:");
        for result in rdr.deserialize() {
            let accident: AccidentInfo = result?;
            println!("{:?}", accident);
            accident_count += 1;
        }
        println!("Total accidents: {}", accident_count);
    } else {
        println!("No accident records found.");
    }
    Ok(())
}

pub async fn generate_detailed_report() {
    println!("\nDetailed Report Menu:");
    println!("1. Display report summary with all data counts");
    println!("2. Show congestion data with heatmap");
    println!("3. Show traffic event data with scatterplot");
    print!("Enter your choice: ");
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    let choice = input.trim().parse::<u32>().unwrap_or(0);
    match choice {
        1 => {
            if let Err(e) = generate_report_summary() {
                eprintln!("Error generating report summary: {}", e);
            }
        }
        2 => {
            if let Err(e) = show_congestion_heatmap() {
                eprintln!("Error generating congestion heatmap: {}", e);
            }
        }
        3 => {
            if let Err(e) = show_traffic_events() {
                eprintln!("Error displaying traffic events: {}", e);
            }
        }
        _ => {
            println!("Invalid choice.");
        }
    }
}

pub async fn run_cli() {
    loop {
        println!("\nTraffic Monitoring System Admin CLI");
        println!("1. Display Congestion Alerts");
        println!("2. Display Light Adjustments");
        println!("3. Display Traffic Data");
        println!("4. Manually Adjust Traffic Light Phase Duration");
        println!("5. Generate Detailed Report");
        println!("6. Exit");
        print!("Enter your choice: ");
        stdout().flush().unwrap();
        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let choice = input.trim().parse::<u32>().unwrap_or(0);
        match choice {
            1 => {
                if let Err(e) = show_congestion_alerts() {
                    eprintln!("Error displaying congestion alerts: {}", e);
                }
            }
            2 => {
                if let Err(e) = show_light_adjustments() {
                    eprintln!("Error displaying light adjustments: {}", e);
                }
            }
            3 => {
                if let Err(e) = show_traffic_data() {
                    eprintln!("Error displaying traffic data: {}", e);
                }
            }
            4 => {
                print!("Enter Intersection ID to adjust (x,y): ");
                stdout().flush().unwrap();
                let mut id_input = String::new();
                stdin().read_line(&mut id_input).unwrap();
                let trimmed_id = id_input.trim();
                let intersection_id = format!("IntersectionId({})", trimmed_id);
                print!("Enter new phase duration (seconds): ");
                stdout().flush().unwrap();
                let mut dur_input = String::new();
                stdin().read_line(&mut dur_input).unwrap();
                let new_duration = dur_input.trim().parse::<u32>().unwrap_or(5);
                match adjust_traffic_light_phase(intersection_id.clone(), new_duration) {
                    Ok(_) => println!(
                        "Adjustment message sent for intersection {}",
                        intersection_id
                    ),
                    Err(e) => eprintln!("Error sending adjustment: {}", e),
                }
            }
            5 => {
                generate_detailed_report().await;
            }
            6 => {
                println!("Exiting CLI.");
                break;
            }
            _ => {
                println!("Invalid choice. Try again.");
            }
        }
    }
}

// Publishes a manual traffic light phase adjustment to the "light_adjustments" queue.
pub fn adjust_traffic_light_phase(intersection_id: String, new_duration: u32) -> AmiquipResult<()> {
    let mut connection = Connection::insecure_open(AMQP_URL)?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);
    let adjustment = LightAdjustment {
        timestamp: current_timestamp(),
        intersection_id,
        add_seconds_green: new_duration,
    };
    let payload = serde_json::to_string(&adjustment).unwrap();
    exchange.publish(Publish::new(payload.as_bytes(), QUEUE_LIGHT_ADJUSTMENTS))?;
    connection.close()
}

use crate::shared_data::current_timestamp;
use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result as AmiquipResult,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct CongestionAlertRecord {
    pub timestamp: u64,
    pub intersection: Option<String>,
    pub message: String,
    pub recommended_action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LightAdjustmentRecord {
    pub timestamp: u64,
    pub intersection_id: String,
    pub add_seconds_green: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrafficDataRecord {
    pub timestamp: u64,
    pub raw_data: String,
}

/// Generic helper to log a record to a CSV file.
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

pub fn log_congestion_alert(record: CongestionAlertRecord) {
    if let Err(e) = log_to_csv("congestion_alerts.csv", &record) {
        eprintln!("Error logging congestion alert: {}", e);
    }
}

pub fn log_light_adjustment(record: LightAdjustmentRecord) {
    if let Err(e) = log_to_csv("light_adjustments.csv", &record) {
        eprintln!("Error logging light adjustment: {}", e);
    }
}

pub fn log_traffic_data(record: TrafficDataRecord) {
    if let Err(e) = log_to_csv("traffic_data.csv", &record) {
        eprintln!("Error logging traffic data: {}", e);
    }
}

/// Listens to the "congestion_alerts" queue and logs each incoming record.
pub async fn listen_congestion_alerts() -> AmiquipResult<()> {
    tokio::task::spawn_blocking(|| -> AmiquipResult<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let _exchange = Exchange::direct(&channel);
        let queue = channel.queue_declare("congestion_alerts", QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        println!("Listening for congestion alerts...");
        for message in consumer.receiver() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let ts = current_timestamp();
                    if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                        let mut record: CongestionAlertRecord = serde_json::from_str(json_str)
                            .unwrap_or(CongestionAlertRecord {
                                timestamp: ts,
                                intersection: None,
                                message: json_str.to_string(),
                                recommended_action: "".to_string(),
                            });
                        record.timestamp = ts;
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

/// Listens to the "light_adjustments" queue and logs each incoming record.
pub async fn listen_light_adjustments() -> AmiquipResult<()> {
    tokio::task::spawn_blocking(|| -> AmiquipResult<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let _exchange = Exchange::direct(&channel);
        let queue = channel.queue_declare("light_adjustments", QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        println!("Listening for light adjustments...");
        for message in consumer.receiver() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let ts = current_timestamp();
                    if let Ok(json_str) = std::str::from_utf8(&delivery.body) {
                        let mut record: LightAdjustmentRecord = serde_json::from_str(json_str)
                            .unwrap_or(LightAdjustmentRecord {
                                timestamp: ts,
                                intersection_id: "unknown".to_string(),
                                add_seconds_green: 0,
                            });
                        record.timestamp = ts;
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

/// Listens to the "traffic_data" queue and logs each incoming record.
pub async fn listen_traffic_data() -> AmiquipResult<()> {
    tokio::task::spawn_blocking(|| -> AmiquipResult<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let _exchange = Exchange::direct(&channel);
        let queue = channel.queue_declare("traffic_data", QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        println!("Listening for traffic data...");
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

/// Reads and displays records from "congestion_alerts.csv".
pub fn show_congestion_alerts() -> Result<(), Box<dyn Error>> {
    let file = File::open("congestion_alerts.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    println!("Congestion Alerts:");
    for result in rdr.deserialize() {
        let record: CongestionAlertRecord = result?;
        println!("{:?}", record);
    }
    Ok(())
}

/// Reads and displays records from "light_adjustments.csv".
pub fn show_light_adjustments() -> Result<(), Box<dyn Error>> {
    let file = File::open("light_adjustments.csv")?;
    let mut rdr = csv::Reader::from_reader(file);
    println!("Light Adjustments:");
    for result in rdr.deserialize() {
        let record: LightAdjustmentRecord = result?;
        println!("{:?}", record);
    }
    Ok(())
}

/// Reads and displays records from "traffic_data.csv".
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

/// Publishes a manual traffic light phase adjustment to the "light_adjustments" queue.
pub fn adjust_traffic_light_phase(intersection_id: String, new_duration: u32) -> AmiquipResult<()> {
    let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);
    let adjustment = LightAdjustmentRecord {
        timestamp: current_timestamp(),
        intersection_id,
        add_seconds_green: new_duration,
    };
    let payload = serde_json::to_string(&adjustment).unwrap();
    exchange.publish(Publish::new(payload.as_bytes(), "light_adjustments"))?;
    connection.close()
}

/// Generates a simple report by counting the number of records in each CSV file.
pub fn generate_report() -> Result<(), Box<dyn Error>> {
    println!("Generating Report...");
    let congestion_count = count_csv_records("congestion_alerts.csv")?;
    let light_adjustments_count = count_csv_records("light_adjustments.csv")?;
    let traffic_data_count = count_csv_records("traffic_data.csv")?;
    println!("Report Summary:");
    println!("Congestion Alerts: {} records", congestion_count);
    println!("Light Adjustments: {} records", light_adjustments_count);
    println!("Traffic Data: {} records", traffic_data_count);
    Ok(())
}

fn count_csv_records(filename: &str) -> Result<usize, Box<dyn Error>> {
    let file = File::open(filename)?;
    let mut rdr = csv::Reader::from_reader(file);
    let count = rdr.deserialize::<serde_json::Value>().count();
    Ok(count)
}

/// Provides a simple CLI for admin operations.
pub async fn run_cli() {
    use std::io::{stdin, stdout, Write};
    loop {
        println!("\nTraffic Monitoring System Admin CLI");
        println!("1. Display Congestion Alerts");
        println!("2. Display Light Adjustments");
        println!("3. Display Traffic Data");
        println!("4. Manually Adjust Traffic Light Phase Duration");
        println!("5. Generate Report");
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
                print!("Enter Intersection ID to adjust: ");
                stdout().flush().unwrap();
                let mut id_input = String::new();
                stdin().read_line(&mut id_input).unwrap();
                let intersection_id = id_input.trim().to_string();
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
                if let Err(e) = generate_report() {
                    eprintln!("Error generating report: {}", e);
                }
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

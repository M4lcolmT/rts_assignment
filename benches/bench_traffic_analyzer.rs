use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};
use std::collections::{HashMap, HashSet, VecDeque};

use rts_assignment::c2_tp063881::traffic_analyzer::{
    analyze_traffic_data, predict_future_traffic_weighted, HistoricalData,
};
use rts_assignment::shared_data::TrafficData;

/// Generates dummy TrafficData for a given batch size.
/// Each intersection is named "Intersection_{i}" with simulated occupancy and waiting times.
fn generate_dummy_traffic_data_batch(batch_size: usize) -> TrafficData {
    let mut lane_occupancy = HashMap::new();
    let mut intersection_congestion = HashMap::new();
    let mut intersection_waiting_time = HashMap::new();
    // For simplicity, accident lanes and vehicle_data are empty.
    let accident_lanes = HashSet::new();
    let vehicle_data = vec![];

    for i in 0..batch_size {
        let key = format!("Intersection_{}", i);
        // Occupancy ranges roughly from 0.5 to 0.9.
        let occupancy = 0.5 + ((i % 10) as f64) * 0.04;
        // Waiting time ranges roughly from 3.0 to 10.0 seconds.
        let waiting_time = 3.0 + ((i % 10) as f64) * 0.7;
        lane_occupancy.insert(key.clone(), occupancy);
        intersection_congestion.insert(key.clone(), occupancy);
        intersection_waiting_time.insert(key, waiting_time);
    }

    TrafficData {
        lane_occupancy,
        accident_lanes,
        intersection_congestion,
        intersection_waiting_time,
        vehicle_data,
    }
}

/// Generates dummy HistoricalData from the given TrafficData.
/// For each intersection, creates a two-entry history with slight variations.
fn generate_dummy_historical_data(data: &TrafficData) -> HistoricalData {
    let mut historical = HistoricalData::new(10);
    for (key, &occ) in data.intersection_congestion.iter() {
        let mut occ_history = VecDeque::new();
        occ_history.push_back(occ * 0.95);
        occ_history.push_back(occ * 1.05);
        historical
            .occupancy_history
            .insert(key.clone(), occ_history);
    }
    for (key, &wt) in data.intersection_waiting_time.iter() {
        let mut wt_history = VecDeque::new();
        wt_history.push_back(wt * 0.9);
        wt_history.push_back(wt * 1.1);
        historical
            .waiting_time_history
            .insert(key.clone(), wt_history);
    }
    historical
}

/// Benchmarks analyze_traffic_data and predict_future_traffic_weighted
/// for different batch sizes (e.g., 50, 100, and 200 intersections).
fn bench_batch_traffic_data(c: &mut Criterion) {
    let batch_sizes = [50, 100, 200];

    // Create a benchmark group and configure it for a linear summary (for plots).
    let mut group = c.benchmark_group("Traffic_Data_Batch_Benchmarks");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Linear));

    for &batch in batch_sizes.iter() {
        let traffic_data = generate_dummy_traffic_data_batch(batch);
        let historical = generate_dummy_historical_data(&traffic_data);

        // Benchmark analyze_traffic_data.
        group.bench_with_input(
            BenchmarkId::new("analyze_traffic_data", batch),
            &batch,
            |b, &_batch| {
                b.iter(|| {
                    let alerts = analyze_traffic_data(black_box(&traffic_data));
                    black_box(alerts);
                });
            },
        );

        // Benchmark predict_future_traffic_weighted.
        group.bench_with_input(
            BenchmarkId::new("predict_future_traffic_weighted", batch),
            &batch,
            |b, &_batch| {
                b.iter(|| {
                    let predicted = predict_future_traffic_weighted(
                        black_box(&traffic_data),
                        black_box(&historical),
                        black_box(0.7),
                    );
                    black_box(predicted);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, bench_batch_traffic_data);
criterion_main!(benches);

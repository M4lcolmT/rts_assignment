use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};
use rts_assignment::c1_tp063879::intersections::create_intersections;
use rts_assignment::c1_tp063879::lanes::create_lanes;
use rts_assignment::c1_tp063879::simulation::{collect_traffic_data, spawn_vehicle};
use std::sync::{Arc, Mutex};
use std::vec;

fn bench_spawn_vehicle_batches(c: &mut Criterion) {
    let intersections = Arc::new(Mutex::new(create_intersections()));
    let lanes = Arc::new(Mutex::new(create_lanes()));

    let lanes_guard = lanes.lock().unwrap();
    let intersections_guard = intersections.lock().unwrap();
    let traffic_data = collect_traffic_data(&lanes_guard, &intersections_guard, vec![]);
    drop(lanes_guard);
    drop(intersections_guard);

    let batch_sizes = [50, 100, 200];

    let mut group = c.benchmark_group("spawn_vehicle_batch");

    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Linear));

    for &batch_size in &batch_sizes {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &size| {
                b.iter(|| {
                    // In each iteration, spawn 'size' vehicles
                    let mut next_vehicle_id = 1;
                    for _ in 0..size {
                        let result = spawn_vehicle(
                            &intersections,
                            &lanes,
                            &traffic_data,
                            &mut next_vehicle_id,
                        );
                        black_box(result);
                    }
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_spawn_vehicle_batches);
criterion_main!(benches);

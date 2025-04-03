use criterion::{criterion_group, criterion_main, Criterion};
use rts_assignment::monitoring::traffic_monitoring_system::{
    listen_congestion_alerts, listen_light_adjustments, listen_traffic_data, listen_traffic_event,
};
use std::time::Duration;
use tokio;

fn bench_monitoring_system(c: &mut Criterion) {
    // Create a single runtime
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("monitoring_system");
    // Limit the time spent on each benchmark
    group.measurement_time(Duration::from_secs(13));
    group.sample_size(100);

    // Add timeouts to each function
    group.bench_function("listen_congestion_alerts", |b| {
        b.iter(|| {
            rt.block_on(async {
                tokio::select! {
                    _ = listen_congestion_alerts() => {},
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {},
                }
            })
        });
    });

    group.bench_function("listen_light_adjustments", |b| {
        b.iter(|| {
            rt.block_on(async {
                tokio::select! {
                    _ = listen_light_adjustments() => {},
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {},
                }
            })
        });
    });

    group.bench_function("listen_traffic_data", |b| {
        b.iter(|| {
            rt.block_on(async {
                tokio::select! {
                    _ = listen_traffic_data() => {},
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {},
                }
            })
        });
    });

    group.bench_function("listen_traffic_event", |b| {
        b.iter(|| {
            rt.block_on(async {
                tokio::select! {
                    _ = listen_traffic_event() => {},
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {},
                }
            })
        });
    });

    group.finish();
}

criterion_group!(benches, bench_monitoring_system);
criterion_main!(benches);

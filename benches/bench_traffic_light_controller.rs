// benches/bench_update.rs
use criterion::{
    black_box, AxisScale, Criterion, PlotConfiguration, criterion_group, criterion_main,
};
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DummyIntersectionId(u32);

#[derive(Clone, Debug)]
struct DummyIntersection {
    id: DummyIntersectionId,
    control: IntersectionControl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum IntersectionControl {
    TrafficLight,
}

// Simplified TrafficLightPhase used in the benchmark.
#[derive(Debug)]
struct TrafficLightPhase {
    green_lanes: Vec<String>,
    duration: u64, // seconds
}

// A simplified IntersectionController with the update function.
#[derive(Debug)]
struct IntersectionController {
    intersection: DummyIntersection,
    phases: Vec<TrafficLightPhase>,
    current_phase_index: usize,
    elapsed_in_phase: u64,
    emergency_override: Option<Vec<String>>,
}

impl IntersectionController {
    fn new(
        intersection: DummyIntersection,
        phases: Vec<TrafficLightPhase>,
        _all_lanes: Vec<String>,
    ) -> Self {
        Self {
            intersection,
            phases,
            current_phase_index: 0,
            elapsed_in_phase: 0,
            emergency_override: None,
        }
    }

    // The update function we're benchmarking.
    fn update(&mut self) {
        if self.emergency_override.is_some() {
            return;
        }
        self.elapsed_in_phase += 1;
        let current_phase = &self.phases[self.current_phase_index];
        if self.elapsed_in_phase >= current_phase.duration {
            self.elapsed_in_phase = 0;
            self.current_phase_index = (self.current_phase_index + 1) % self.phases.len();
            self.apply_current_phase();
        }
    }

    // A dummy apply_current_phase that does minimal work to avoid I/O overhead.
    fn apply_current_phase(&self) {
        // No printing or heavy computation here.
    }
}

// Helper function to create a dummy IntersectionController with a given number of lanes.
fn create_update_controller(num_lanes: usize) -> IntersectionController {
    let intersection_id = DummyIntersectionId(1);
    let intersection = DummyIntersection {
        id: intersection_id.clone(),
        control: IntersectionControl::TrafficLight,
    };
    // Create lane names: "lane0", "lane1", ..., "lane{num_lanes-1}".
    let lane_names: Vec<String> = (0..num_lanes)
        .map(|i| format!("lane{}", i))
        .collect();
    // Create a single phase with these lane names as green lanes and a fixed duration.
    let phase = TrafficLightPhase {
        green_lanes: lane_names.clone(),
        duration: 8, // Example duration.
    };
    IntersectionController::new(intersection, vec![phase], lane_names)
}

fn bench_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("update_function");
    
    // Increase sample size and extend measurement time for more detailed stats.
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(2));
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Linear));
    
    // Benchmark for controllers with 50, 100, and 200 lanes.
    for &size in [50, 100, 200].iter() {
        group.bench_function(format!("size_{}", size), |b| {
            let mut controller = create_update_controller(size);
            b.iter(|| {
                // Call update() repeatedly.
                controller.update();
                black_box(&controller);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_update);
criterion_main!(benches);

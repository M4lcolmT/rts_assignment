// benches/bench_lane_is_green.rs

use criterion::{
    black_box, AxisScale, Criterion, PlotConfiguration, criterion_group, criterion_main,
};
use std::collections::HashMap;
use std::time::Duration;

// Dummy types to mimic your traffic light controller types
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

#[derive(Clone, Debug)]
struct DummyLane {
    name: String,
    from: DummyIntersectionId,
    to: DummyIntersectionId,
}

// A simplified TrafficLightPhase; for this benchmark, we assume all lanes are green.
#[derive(Debug)]
struct TrafficLightPhase {
    green_lanes: Vec<String>,
    duration: u64,
}

// A simplified IntersectionController
#[derive(Debug)]
struct IntersectionController {
    intersection: DummyIntersection,
    phases: Vec<TrafficLightPhase>,
    current_phase_index: usize,
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
        }
    }
}

// The TrafficLightController with the is_lane_green function.
#[derive(Debug)]
struct TrafficLightController {
    controllers: HashMap<DummyIntersectionId, IntersectionController>,
}

impl TrafficLightController {
    pub fn is_lane_green(&self, intersection_id: DummyIntersectionId, lane_name: &str) -> bool {
        if let Some(ctrl) = self.controllers.get(&intersection_id) {
            // In this dummy example, we ignore emergency override.
            let current_phase = &ctrl.phases[ctrl.current_phase_index];
            return current_phase.green_lanes.contains(&lane_name.to_string());
        }
        // Default to green if the intersection is not controlled.
        true
    }
}

// Helper function to create a dummy controller with a given number of lanes.
fn create_controller(num_lanes: usize) -> TrafficLightController {
    let intersection_id = DummyIntersectionId(1);
    let intersection = DummyIntersection {
        id: intersection_id.clone(),
        control: IntersectionControl::TrafficLight,
    };
    // Create dummy lanes: lane0, lane1, ..., lane{num_lanes-1}.
    let lanes: Vec<DummyLane> = (0..num_lanes)
        .map(|i| DummyLane {
            name: format!("lane{}", i),
            from: intersection_id.clone(),
            to: DummyIntersectionId(2),
        })
        .collect();
    let all_lane_names: Vec<String> = lanes.iter().map(|l| l.name.clone()).collect();
    // For this benchmark, mark all lanes as green.
    let phase = TrafficLightPhase {
        green_lanes: all_lane_names.clone(),
        duration: 8,
    };
    let controller = IntersectionController::new(intersection, vec![phase], all_lane_names);
    let mut controllers = HashMap::new();
    controllers.insert(DummyIntersectionId(1), controller);
    TrafficLightController { controllers }
}

// Benchmark function using Criterion
fn bench_is_lane_green(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_lane_green");
    
    // Increase sample size and measurement/warm-up times for more detailed stats.
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(5));
    group.warm_up_time(Duration::from_secs(2));
    group.plot_config(
        PlotConfiguration::default().summary_scale(AxisScale::Linear),
    );
    
    // Run benchmarks for dataset sizes 50, 100, and 200.
    for &size in [50, 100, 200].iter() {
        group.bench_function(format!("size_{}", size), |b| {
            // Create a dummy controller for each size.
            let controller = create_controller(size);
            let intersection_id = DummyIntersectionId(1);
            let lane_names: Vec<String> = (0..size)
                .map(|i| format!("lane{}", i))
                .collect();
            b.iter(|| {
                // For each lane, call is_lane_green.
                for lane in &lane_names {
                    black_box(controller.is_lane_green(intersection_id.clone(), lane));
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_is_lane_green);
criterion_main!(benches);

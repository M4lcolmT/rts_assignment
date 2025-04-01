use crate::simulation_engine::intersections::IntersectionId;
use crate::simulation_engine::lanes::Lane;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

#[derive(Debug)]
struct State {
    cost: f64,
    intersection: IntersectionId,
}

// Reverse ordering to use BinaryHeap as a min-heap.
impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Notice the flip here: lower cost gets higher priority.
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for State {}

// Use Dijkstra's algorithm to find the shortest route of Lanes from `entry` to `exit`.
// Returns None if no path exists.
pub fn generate_shortest_lane_route(
    lanes: &[Lane],
    entry: IntersectionId,
    exit: IntersectionId,
) -> Option<Vec<Lane>> {
    // Build an adjacency list: each intersection -> all lanes going *out* from it.
    let mut graph: HashMap<IntersectionId, Vec<&Lane>> = HashMap::new();
    for lane in lanes {
        // Build adjacency from lane.from
        graph.entry(lane.from).or_default().push(lane);

        // Also ensure lane.to is present in the map (even if no outgoing lanes from there)
        graph.entry(lane.to).or_default();
    }

    // Distance from `entry` to each intersection
    let mut dist: HashMap<IntersectionId, f64> = HashMap::new();
    // For backtracking: store which Lane got us to a given intersection
    let mut prev: HashMap<IntersectionId, &Lane> = HashMap::new();

    // Min-heap for Dijkstra
    let mut heap = BinaryHeap::new();

    // Initialize the starting point
    dist.insert(entry, 0.0);
    heap.push(State {
        cost: 0.0,
        intersection: entry,
    });

    // Dijkstra's main loop
    while let Some(State { cost, intersection }) = heap.pop() {
        // If we've reached the exit, we can stop
        if intersection == exit {
            break;
        }

        // If there's already a better path to `intersection`, skip
        if let Some(&best_so_far) = dist.get(&intersection) {
            if cost > best_so_far {
                continue;
            }
        }

        // Explore outgoing lanes from this intersection
        if let Some(neighbors) = graph.get(&intersection) {
            for &lane in neighbors {
                let next = lane.to;
                let next_cost = cost + lane.length_meters;

                if next_cost < *dist.get(&next).unwrap_or(&f64::INFINITY) {
                    dist.insert(next, next_cost);
                    prev.insert(next, lane);
                    heap.push(State {
                        cost: next_cost,
                        intersection: next,
                    });
                }
            }
        }
    }

    // If the exit has no recorded distance, we never reached it
    if !dist.contains_key(&exit) {
        println!("No route found from {:?} to {:?}", entry, exit);
        return None;
    }

    // Reconstruct the path from exit back to entry
    let mut route: Vec<Lane> = Vec::new();
    let mut current = exit;
    while current != entry {
        if let Some(&lane) = prev.get(&current) {
            route.push(lane.clone());
            current = lane.from;
        } else {
            // Should never happen if dist[exit] was set
            return None;
        }
    }
    route.reverse();
    Some(route)
}

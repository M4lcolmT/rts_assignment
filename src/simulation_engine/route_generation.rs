// route_generation.rs
//
// This module provides functions for generating and replanning routes dynamically.
// It uses a randomized breadth-first search (BFS) algorithm to find a path
// from a randomly chosen entry to a randomly chosen exit, or from a given
// starting intersection (when replanning) to a random exit. The routes generated
// are sequences of IntersectionIds that vehicles will follow.

use crate::simulation_engine::grid::TrafficGrid;
use crate::simulation_engine::intersection::IntersectionId;
use rand::seq::{IndexedRandom, SliceRandom};
use rand::Rng;
use std::collections::{HashMap, VecDeque};

/// Generates a random route for a vehicle.
/// The function selects a random entry intersection and a random exit intersection
/// from the grid and then finds a path between them using a randomized BFS.
/// Returns a vector of IntersectionIds representing the route.
/// If no route is found, returns an empty vector.
pub fn generate_random_route(grid: &TrafficGrid) -> Vec<IntersectionId> {
    let mut rng = rand::rng();

    // Collect available entry and exit intersections from the grid.
    let entries: Vec<_> = grid
        .intersections
        .values()
        .filter(|intersection| intersection.is_entry)
        .collect();
    let exits: Vec<_> = grid
        .intersections
        .values()
        .filter(|intersection| intersection.is_exit)
        .collect();

    if entries.is_empty() || exits.is_empty() {
        return Vec::new();
    }

    // Randomly select an entry intersection.
    let start = entries.choose(&mut rng).unwrap().id;
    // Randomly select an exit intersection that is different from the start.
    let target = loop {
        let candidate = exits.choose(&mut rng).unwrap().id;
        if candidate != start {
            break candidate;
        }
    };

    bfs_random(grid, start, target).unwrap_or_else(|| vec![start])
}

/// Replans a route for a vehicle starting at the given intersection.
/// The function selects a random exit intersection (different from current)
/// and computes a path from `current` to that exit using a randomized BFS.
/// Returns a new route as a vector of IntersectionIds. If no route is found,
/// it returns a vector containing only the current intersection.
pub fn replan_route(grid: &TrafficGrid, current: IntersectionId) -> Vec<IntersectionId> {
    let mut rng = rand::rng();
    // Get all intersections marked as exit.
    let exits: Vec<_> = grid
        .intersections
        .values()
        .filter(|intersection| intersection.is_exit)
        .collect();
    if exits.is_empty() {
        return vec![current];
    }

    // Randomly choose an exit that is not the current intersection.
    let target = loop {
        let candidate = exits.choose(&mut rng).unwrap().id;
        if candidate != current {
            break candidate;
        }
    };

    bfs_random(grid, current, target).unwrap_or_else(|| vec![current])
}

/// Helper function: randomized breadth-first search (BFS)
/// to find a path from `start` to `target`. The neighbors are shuffled for randomness.
/// Returns an Option containing a vector of IntersectionIds representing the path.
fn bfs_random(
    grid: &TrafficGrid,
    start: IntersectionId,
    target: IntersectionId,
) -> Option<Vec<IntersectionId>> {
    let mut queue = VecDeque::new();
    queue.push_back(start);
    let mut came_from: HashMap<IntersectionId, IntersectionId> = HashMap::new();
    came_from.insert(start, start); // Mark the start with itself.

    while let Some(current) = queue.pop_front() {
        if current == target {
            // Reconstruct path from target back to start.
            let mut path = Vec::new();
            let mut cur = current;
            while cur != start {
                path.push(cur);
                cur = came_from.get(&cur).cloned().unwrap();
            }
            path.push(start);
            path.reverse();
            return Some(path);
        }

        // Get neighbors from the current intersection.
        if let Some(intersection) = grid.get_intersection(&current) {
            let mut neighbors = intersection.connected.clone();
            // Shuffle neighbors to introduce randomness.
            neighbors.shuffle(&mut rand::rng());
            for neighbor in neighbors {
                if !came_from.contains_key(&neighbor) {
                    came_from.insert(neighbor, current);
                    queue.push_back(neighbor);
                }
            }
        }
    }
    None
}

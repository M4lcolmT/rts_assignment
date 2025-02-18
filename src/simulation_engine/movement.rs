use crate::simulation_engine::grid::TrafficGrid;
use crate::simulation_engine::intersections::{IntersectionControl, IntersectionId, LightState};
use crate::simulation_engine::lanes::Lane;
use crate::simulation_engine::vehicles::Vehicle;

/// Attempts to retrieve a mutable reference to the lane that connects two intersections.
/// Returns `Some(&mut Lane)` if a matching lane is found, or `None` otherwise.
pub fn get_lane_between<'a>(
    from: &IntersectionId,
    to: &IntersectionId,
    lanes: &'a mut Vec<Lane>,
) -> Option<&'a mut Lane> {
    lanes
        .iter_mut()
        .find(|lane| &lane.from == from && &lane.to == to)
}

/// Advances a vehicle along its planned route.
///
/// # Arguments
///
/// * `vehicle` - A mutable reference to the vehicle to be moved.
/// * `route` - A mutable vector of intersections representing the vehicle's planned route.  
///             The first element is the current intersection and the next element is the next destination.
/// * `grid` - Reference to the traffic grid (used for checking intersection states).
/// * `lanes` - Mutable reference to the collection of lanes (used to update occupancy).
///
/// # Returns
///
/// * `true` if the vehicle successfully moved to the next intersection.
/// * `false` if the vehicle could not move (due to a red light, lane congestion, or no available lane).
///
/// # Behavior
///
/// - For non-emergency vehicles:
///     - The function checks the lane's available space (using vehicle length vs. lane length).
///     - If the current intersection is controlled by a traffic light and its state is Red, the vehicle waits.
/// - For emergency vehicles:
///     - They ignore red lights and capacity limits; the lane’s emergency flag is set during movement.
pub fn advance_vehicle(
    vehicle: &mut Vehicle,
    route: &mut Vec<IntersectionId>,
    grid: &TrafficGrid,
    lanes: &mut Vec<Lane>,
) -> bool {
    // If the route has less than 2 nodes, the vehicle is either at destination or no valid route exists.
    if route.len() < 2 {
        return false;
    }

    // The current intersection is the first element; the next intersection is the second.
    let current = route[0];
    let next = route[1];

    // Retrieve the lane connecting the current intersection to the next.
    if let Some(lane) = get_lane_between(&current, &next, lanes) {
        // For non-emergency vehicles, check if the lane has enough free space.
        if !vehicle.is_emergency && !lane.can_add_vehicle(vehicle) {
            return false; // The lane is congested.
        }

        // For non-emergency vehicles, check if the current intersection's traffic light is red.
        if !vehicle.is_emergency {
            if let Some(intersection) = grid.get_intersection(&current) {
                if intersection.control == IntersectionControl::TrafficLight {
                    if let Some(light_state) = intersection.light_state {
                        if light_state == LightState::Red {
                            return false; // Vehicle must wait for a green light.
                        }
                    }
                }
            }
        }

        // Proceed to "move" the vehicle:
        // If the vehicle is emergency, set the lane flag so that other vehicles yield.
        if vehicle.is_emergency {
            lane.has_emergency_vehicle = true;
        } else {
            lane.add_vehicle(vehicle);
        }

        // Simulate the vehicle traversing the lane.
        // (In a full simulation, you might include a time delay based on lane length and vehicle speed.)
        // For now, we assume immediate traversal.

        // Once the vehicle "arrives" at the next intersection, remove it from the lane occupancy.
        if vehicle.is_emergency {
            lane.has_emergency_vehicle = false;
        } else {
            lane.remove_vehicle(vehicle);
        }

        // Remove the current intersection from the route, indicating the vehicle has advanced.
        // (In a more advanced system, you might update a route index rather than modifying the vector.)
        route.remove(0);

        return true;
    }

    false
}

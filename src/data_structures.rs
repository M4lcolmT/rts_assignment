use std::collections::HashMap;

/// A unique identifier for an intersection, using (row, col) coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntersectionId(pub u8, pub u8);

/// Distinguishes intersections by their functional role in the grid:
/// - Entry: Vehicles can only enter the simulation here (green circles).
/// - Exit: Vehicles can only leave the simulation here (red circles).
/// - Normal: Vehicles do not perform any action (blue circles).
/// - TrafficLight: A controlled intersection with signals (purple circles).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionRole {
    Entry,
    Exit,
    Normal,
    TrafficLight,
}

/// The possible states for a traffic light (used only at intersections with role = TrafficLight).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightState {
    Green,
    Yellow,
    Red,
}

/// Some intersections have a one-way out (e.g., entry points),
/// while others might allow two-way traffic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionDirection {
    /// Traffic flows outward only (typical for an Entry intersection).
    OneWayOut,
    /// Traffic can flow in both directions (typical for traffic lights or exit intersections).
    TwoWay,
}

/// Represents an intersection in the grid.
#[derive(Debug, Clone)]
pub struct Intersection {
    /// The row, col coordinates (e.g., (0,0), (3,2), etc.).
    pub id: IntersectionId,
    /// Whether this intersection is an Entry, Exit, or has a Traffic Light.
    pub role: IntersectionRole,
    /// The current state of the traffic light (if applicable).
    pub light_state: Option<LightState>,
    /// Indicates whether traffic flows only out of this intersection or in both directions.
    pub direction: IntersectionDirection,
    /// A list of adjacent intersections this intersection connects to.
    /// Useful for pathfinding or checking valid next steps.
    pub connected: Vec<IntersectionId>,
}

impl Intersection {
    /// Create a new intersection with a specified role and direction.
    /// By default, non-traffic-light intersections have `light_state = None`.
    pub fn new(
        row: u8,
        col: u8,
        role: IntersectionRole,
        direction: IntersectionDirection,
        connected: Vec<IntersectionId>,
    ) -> Self {
        let light_state = match role {
            IntersectionRole::TrafficLight => Some(LightState::Red),
            _ => None,
        };

        Self {
            id: IntersectionId(row, col),
            role,
            light_state,
            direction,
            connected,
        }
    }
}

/// Different types of vehicles that can appear in the simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleType {
    Car,
    Bus,
    Truck,
    EmergencyVan
}

/// A route is a sequence of intersections from an entry point to an exit point.
/// You can add distance or lane info if needed.
#[derive(Debug, Clone)]
pub struct Route {
    pub path: Vec<IntersectionId>,
}

/// Represents a vehicle traveling through the grid.
#[derive(Debug, Clone)]
pub struct Vehicle {
    /// Unique identifier for the vehicle.
    pub id: u64,
    /// The kind of vehicle (Car, Bus, Truck, etc.).
    pub vehicle_type: VehicleType,
    /// The designated entry intersection where the vehicle starts.
    pub entry_point: IntersectionId,
    /// The designated exit intersection where the vehicle intends to leave.
    pub exit_point: IntersectionId,
    /// Current speed of the vehicle (units per second, for example).
    pub speed: f64,
    /// The planned route for this vehicle, from `entry_point` to `exit_point`.
    pub route: Route,
    /// An index to track the vehicle’s current position in `route.path`.
    pub route_index: usize,
    /// Priority (e.g., 0 for normal, higher for emergency vehicles).
    pub priority: u8,
}

/// Aggregated traffic metrics, typically generated each simulation tick or time step.
#[derive(Debug, Clone)]
pub struct TrafficData {
    /// Total number of vehicles in the simulation.
    pub total_vehicles: usize,
    /// Average delay experienced by vehicles (in seconds).
    pub average_delay: f64,
    /// Additional metrics or stats can be added here.
}

/// Discrete events that can occur during the simulation.
#[derive(Debug, Clone)]
pub enum TrafficEvent {
    /// A vehicle has successfully reached its exit intersection.
    VehicleArrived(u64),
    /// A vehicle has departed an intersection (possibly heading to the next).
    VehicleDeparted(u64),
    /// Two vehicles have collided; store both vehicle IDs.
    Collision(u64, u64),
    /// A general incident, e.g. a stalled vehicle or blocked lane.
    Incident(String),
}

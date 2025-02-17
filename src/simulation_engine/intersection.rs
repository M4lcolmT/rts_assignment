/// Unique identifier for an intersection using (row, col) coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntersectionId(pub i8, pub i8);

/// Represents control at an intersection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionControl {
    Normal,       // Standard intersection without traffic lights
    TrafficLight, // Intersection with traffic light control
}

/// Possible states of a traffic light (if applicable).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightState {
    Green,
    Yellow,
    Red,
}

/// Represents a traffic intersection (node).
#[derive(Debug, Clone)]
pub struct Intersection {
    /// Unique identifier for the intersection.
    pub id: IntersectionId,
    /// Whether vehicles can enter at this intersection.
    pub is_entry: bool,
    /// Whether vehicles can exit at this intersection.
    pub is_exit: bool,
    /// Defines if the intersection has a traffic light or is a normal junction.
    pub control: IntersectionControl,
    /// The current traffic light state (if applicable).
    pub light_state: Option<LightState>,
    /// Connected intersections (adjacent nodes).
    pub connected: Vec<IntersectionId>,
    /// Flag to check if an emergency vehicle is currently in the intersection.
    pub has_emergency_vehicle: bool,
}

impl Intersection {
    /// Creates a new intersection with the given properties.
    pub fn new(
        row: i8,
        col: i8,
        is_entry: bool,
        is_exit: bool,
        control: IntersectionControl,
        connected: Vec<IntersectionId>,
    ) -> Self {
        let light_state = match control {
            IntersectionControl::TrafficLight => Some(LightState::Red),
            _ => None,
        };

        Self {
            id: IntersectionId(row, col),
            is_entry,
            is_exit,
            control,
            light_state,
            connected,
            has_emergency_vehicle: false,
        }
    }

    /// Updates the traffic light state (if the intersection has one).
    pub fn update_light(&mut self) {
        if self.control == IntersectionControl::TrafficLight && !self.has_emergency_vehicle {
            if let Some(state) = self.light_state {
                self.light_state = match state {
                    LightState::Red => Some(LightState::Green),
                    LightState::Green => Some(LightState::Yellow),
                    LightState::Yellow => Some(LightState::Red),
                };
            }
        }
    }

    /// Clears the intersection for emergency vehicles by forcing all vehicles to stop.
    pub fn clear_intersection_for_emergency(&mut self) {
        self.has_emergency_vehicle = true;
        self.light_state = Some(LightState::Green);
    }

    /// Resets the emergency state after the vehicle has passed.
    pub fn reset_emergency_status(&mut self) {
        self.has_emergency_vehicle = false;
    }
}

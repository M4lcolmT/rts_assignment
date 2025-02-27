#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntersectionId(pub i8, pub i8);

/// Represents control at an intersection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionControl {
    Normal,       // Standard intersection without traffic lights
    TrafficLight, // Intersection with traffic light control
}

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
    /// Intersection Name
    pub name: String,
    /// Whether vehicles can enter at this intersection.
    pub is_entry: bool,
    /// Whether vehicles can exit at this intersection.
    pub is_exit: bool,
    /// Defines if the intersection has a traffic light or is a normal junction.
    pub control: IntersectionControl,
    /// The current traffic light state (if applicable).
    pub light_state: Option<LightState>,
    /// Flag to check if an emergency vehicle is currently in the intersection.
    pub has_emergency_vehicle: bool,
}

impl Intersection {
    /// Creates a new intersection with the given properties.
    pub fn new(
        name: String,
        row: i8,
        col: i8,
        is_entry: bool,
        is_exit: bool,
        control: IntersectionControl,
    ) -> Self {
        let light_state = match control {
            IntersectionControl::TrafficLight => Some(LightState::Red),
            _ => None,
        };

        Self {
            id: IntersectionId(row, col),
            name,
            is_entry,
            is_exit,
            control,
            light_state,
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
}

// TODO: remove and put inside traffic light controller
// A helper function to simulate stopping vehicles at an intersection.
// pub fn clear_intersection_for_emergency(intersection: &mut Intersection) {
//     if !intersection.has_emergency_vehicle {
//         println!(
//             "Clearing intersection {:?} for emergency: switching light to Red.",
//             intersection.id
//         );
//     }
//     intersection.light_state = Some(LightState::Red);
//     intersection.has_emergency_vehicle = true;
// }

// TODO: remove and put inside traffic light controller
// A helper function to restore an intersection's traffic light.
// pub fn restore_intersection(intersection: &mut Intersection) {
//     println!(
//         "Intersection {:?} is now restored to normal operation: switching light to Green.",
//         intersection.id
//     );
//     intersection.light_state = Some(LightState::Green); // Restore the light to Green
//     intersection.has_emergency_vehicle = false; // No longer an emergency
//     intersection.update_light(); // Update the light if needed
// }

pub fn create_intersections() -> Vec<Intersection> {
    vec![
        Intersection::new(
            "Intersection 00".to_string(),
            0,
            0,
            true,
            false,
            IntersectionControl::Normal,
        ),
        Intersection::new(
            "Intersection 01".to_string(),
            0,
            1,
            false,
            true,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 02".to_string(),
            0,
            2,
            true,
            false,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 03".to_string(),
            0,
            3,
            false,
            true,
            IntersectionControl::Normal,
        ),
        Intersection::new(
            "Intersection 10".to_string(),
            1,
            0,
            true,
            true,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 11".to_string(),
            1,
            1,
            false,
            false,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 12".to_string(),
            1,
            2,
            false,
            false,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 13".to_string(),
            1,
            3,
            true,
            true,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 20".to_string(),
            2,
            0,
            true,
            true,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 21".to_string(),
            2,
            1,
            false,
            false,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 22".to_string(),
            2,
            2,
            false,
            false,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 23".to_string(),
            2,
            3,
            true,
            true,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 30".to_string(),
            3,
            0,
            false,
            true,
            IntersectionControl::Normal,
        ),
        Intersection::new(
            "Intersection 31".to_string(),
            3,
            1,
            false,
            true,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 32".to_string(),
            3,
            2,
            true,
            false,
            IntersectionControl::TrafficLight,
        ),
        Intersection::new(
            "Intersection 33".to_string(),
            3,
            3,
            true,
            false,
            IntersectionControl::Normal,
        ),
    ]
}

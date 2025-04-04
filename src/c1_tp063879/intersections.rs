#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct IntersectionId(pub i8, pub i8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntersectionControl {
    Normal,       // Standard intersection without traffic lights
    TrafficLight, // Intersection with traffic light control
}
#[derive(Debug, Clone)]
pub struct Intersection {
    pub id: IntersectionId,
    pub name: String,
    pub is_entry: bool,
    pub is_exit: bool,
    // Defines if the intersection has a traffic light or is a normal junction.
    pub control: IntersectionControl,
    waiting_time: f64,
}

impl Intersection {
    pub fn new(
        name: String,
        row: i8,
        col: i8,
        is_entry: bool,
        is_exit: bool,
        control: IntersectionControl,
    ) -> Self {
        Self {
            id: IntersectionId(row, col),
            name,
            is_entry,
            is_exit,
            control,
            waiting_time: 0.0,
        }
    }

    pub fn avg_waiting_time(&self) -> f64 {
        self.waiting_time
    }
}

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

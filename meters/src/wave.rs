use direction::*;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Wave {
    pub leader: bool,
    pub direction: CardinalDirection,
    pub range: i32,
}

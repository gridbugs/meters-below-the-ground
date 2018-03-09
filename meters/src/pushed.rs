use direction::CardinalDirection;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PushedInfo {
    pub direction: CardinalDirection,
    pub distance: i32,
    pub range: i32,
}

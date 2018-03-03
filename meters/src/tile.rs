use direction::CardinalDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    Floor,
    Punch(CardinalDirection),
    Stairs,
    Exit,
    Bullet,
    Larvae,
}

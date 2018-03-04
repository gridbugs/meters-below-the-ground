use direction::CardinalDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tile {
    Player,
    Wall,
    CavernWall,
    Door,
    Floor,
    Punch(CardinalDirection),
    Stairs,
    Exit,
    Bullet,
    Larvae,
    Queen,
}

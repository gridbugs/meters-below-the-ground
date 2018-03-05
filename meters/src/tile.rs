use direction::CardinalDirection;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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
    RailGunShotHorizontal,
    RailGunShotVertical,
    Larvae,
    Queen,
    HealthPickup,
    AmmoPickup,
}

use tile::Tile;
use meter::Meter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TileInfo {
    pub tile: Tile,
    pub depth: i32,
    pub damage_flash: bool,
    pub boss: bool,
    pub health_meter: Option<Meter>,
    pub countdown: Option<i32>,
}

impl TileInfo {
    pub fn new(tile: Tile, depth: i32) -> Self {
        Self {
            tile,
            depth,
            damage_flash: false,
            boss: false,
            health_meter: None,
            countdown: None,
        }
    }
    pub fn with_health(tile: Tile, depth: i32, health_meter: Meter) -> Self {
        Self {
            tile,
            depth,
            damage_flash: false,
            boss: false,
            health_meter: Some(health_meter),
            countdown: None,
        }
    }
}

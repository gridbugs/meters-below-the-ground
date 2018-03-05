use tile::Tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TileInfo {
    pub tile: Tile,
    pub depth: i32,
    pub damaged: bool,
}

impl TileInfo {
    pub fn new(tile: Tile, depth: i32) -> Self {
        Self {
            tile,
            depth,
            damaged: false,
        }
    }
}

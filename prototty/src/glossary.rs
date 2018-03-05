use std::collections::BTreeSet;
use std::fmt::Write;
use prototty::*;
use prototty_common::*;
use meters::tile_info::*;
use meters::tile::*;

use super::render;

const ENTRY_WIDTH: i32 = 12;
const GLOSSARY_WIDTH: i32 = 60;

pub struct GlossaryView {
    scratch: String,
}

impl GlossaryView {
    pub fn new() -> Self {
        Self {
            scratch: String::new(),
        }
    }
}

fn write_tile(buf: &mut String, ch: char, tile_info: TileInfo) -> bool {
    const WIDTH: usize = ENTRY_WIDTH as usize;
    match tile_info.tile {
        Tile::Player => write!(buf, "{} {:2$}", ch, "Player", WIDTH),
        Tile::Larvae => write!(buf, "{} {:2$}", ch, "Larvae", WIDTH),
        Tile::Queen => write!(buf, "{} {:2$}", ch, "Queen", WIDTH),
        Tile::Stairs => write!(buf, "{} {:2$}", ch, "Stairs", WIDTH),
        Tile::Exit => write!(buf, "{} {:2$}", ch, "Exit", WIDTH),
        Tile::Wall => return false,
        Tile::CavernWall => return false,
        Tile::Door => return false,
        Tile::Floor => return false,
        Tile::Punch(_) => return false,
        Tile::Bullet => return false,
    }.unwrap();
    true
}

impl View<BTreeSet<TileInfo>> for GlossaryView {
    fn view<G: ViewGrid>(&mut self, glossary: &BTreeSet<TileInfo>, offset: Coord, depth: i32, grid: &mut G) {
        let mut coord = Coord::new(0, 0);
        for &tile_info in glossary.iter() {
            let (ch, mut info) = render::tile_text(tile_info);
            info.background_colour = None;
            if write_tile(&mut self.scratch, ch, tile_info) {
                TextInfoStringView.view(&(info, &self.scratch), offset + coord, depth, grid);
                self.scratch.clear();
                coord.x += ENTRY_WIDTH;
                if coord.x + ENTRY_WIDTH > GLOSSARY_WIDTH {
                    coord.x = 0;
                    coord.y += 1;
                }
            }
        }
    }
}

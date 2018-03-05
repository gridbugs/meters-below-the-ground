use std::collections::BTreeSet;
use std::fmt::Write;
use prototty::*;
use prototty_common::*;
use meters::tile_info::*;
use meters::tile::*;

use super::render;

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

fn write_tile(stage: &mut String, ch: char, tile_info: TileInfo) -> bool {
    let extra = if tile_info.boss {
        " (boss)"
    } else if tile_info.wounded {
        " (1hp)"
    } else {
        ""
    };
    match tile_info.tile {
        Tile::Player => write!(stage, "{} {}{}", ch, "Player", extra),
        Tile::Larvae => write!(stage, "{} {}{}", ch, "Larvae", extra),
        Tile::Queen => write!(stage, "{} {}{}", ch, "Queen", extra),
        Tile::Stairs => write!(stage, "{} {}", ch, "Stairs"),
        Tile::Exit => write!(stage, "{} {}", ch, "Exit"),
        Tile::HealthPickup => write!(stage, "{} {}", ch, "Meds"),
        Tile::AmmoPickup => write!(stage, "{} {}", ch, "Ammo"),
        Tile::Wall |
        Tile::CavernWall |
        Tile::Door |
        Tile::Floor |
        Tile::Punch(_) |
        Tile::Bullet |
        Tile::RailGunShotHorizontal |
        Tile::RailGunShotVertical => return false,
    }.unwrap();

    true
}

impl View<BTreeSet<TileInfo>> for GlossaryView {
    fn view<G: ViewGrid>(&mut self, glossary: &BTreeSet<TileInfo>, offset: Coord, depth: i32, grid: &mut G) {
        let mut coord = Coord::new(0, 0);
        for &tile_info in glossary.iter() {
            let (ch, info) = render::tile_text(tile_info);
            self.scratch.clear();
            if write_tile(&mut self.scratch, ch, tile_info) {
                let len = self.scratch.chars().count() as i32 + 2;
                let mut next_x = coord.x + len;
                if next_x > GLOSSARY_WIDTH {
                    coord.x = 0;
                    coord.y += 1;
                    next_x = len;
                }
                TextInfoStringView.view(&(info, &self.scratch), offset + coord, depth, grid);

                coord.x = next_x;
            }
        }
    }
}

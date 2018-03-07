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
    match tile_info.tile {
        Tile::Player => write!(stage, "{} {}", ch, "Player"),
        Tile::Egg => write!(stage, "{} {}", ch, "Egg"),
        Tile::Larvae => write!(stage, "{} {}", ch, "Larvae"),
        Tile::Chrysalis => write!(stage, "{} {}", ch, "Chrysalis"),
        Tile::Aracnoid => write!(stage, "{} {}", ch, "Aracnoid"),
        Tile::Beetoid => write!(stage, "{} {}", ch, "Beetoid"),
        Tile::SuperEgg => write!(stage, "{} {}", ch, "Super Egg"),
        Tile::Queen => write!(stage, "{} {}", ch, "Queen"),
        Tile::Stairs => write!(stage, "{} {}", ch, "Stairs"),
        Tile::Exit => write!(stage, "{} {}", ch, "Exit"),
        Tile::HealthPickup => write!(stage, "{} {}", ch, "Meds"),
        Tile::AmmoPickup => write!(stage, "{} {}", ch, "Quadgun Ammo"),
        Tile::RailGunAmmoPickup => write!(stage, "{} {}", ch, "Railgun Ammo"),
        Tile::KevlarPickup => write!(stage, "{} {}", ch, "Armour Shard"),
        Tile::Wall
        | Tile::CavernWall
        | Tile::Door
        | Tile::Floor
        | Tile::Punch(_)
        | Tile::Bullet
        | Tile::RailGunShotHorizontal
        | Tile::RailGunShotVertical => return false,
    }.unwrap();

    if tile_info.boss {
        write!(stage, " (boss)").unwrap();
    } else {
        if let Some(1) = tile_info.countdown {
            match tile_info.tile {
                Tile::Egg |
                    Tile::Chrysalis |
                    Tile::SuperEgg => {
                        write!(stage, " (hatching)").unwrap();
                        return true;
                    }
                _ => (),
            }
        }
        if let Some(health_meter) = tile_info.health_meter {
            if tile_info.tile != Tile::Player && health_meter.value < health_meter.max {
                write!(stage, " ({}hp)", health_meter.value).unwrap();
            }
        }
    }

    true
}

impl View<BTreeSet<TileInfo>> for GlossaryView {
    fn view<G: ViewGrid>(
        &mut self,
        glossary: &BTreeSet<TileInfo>,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
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

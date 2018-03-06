use meters::tile::Tile;
use meters::tile_info::TileInfo;
use prototty::*;
use prototty_common::*;
use direction::CardinalDirection;

pub fn render_when_non_visible(tile: Tile) -> bool {
    match tile {
        Tile::Player | Tile::Punch(_) | Tile::Larvae | Tile::Queen | Tile::Bullet | Tile::RailGunShotHorizontal |
            Tile::RailGunShotVertical => false,
        Tile::Wall
        | Tile::CavernWall
        | Tile::Door
        | Tile::Floor
        | Tile::Stairs
        | Tile::Exit
        | Tile::AmmoPickup
        | Tile::HealthPickup
        | Tile::KevlarPickup
        | Tile::RailGunAmmoPickup => true,
    }
}

pub fn tile_text(tile_info: TileInfo) -> (char, TextInfo) {
    let (ch, mut text_info) = match tile_info.tile {
        Tile::Player => (
            '@',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(255, 255, 0)),
        ),
        Tile::Wall => (
            '#',
            TextInfo::default()
                .foreground_colour(Rgb24::new(0, 0, 0))
                .background_colour(Rgb24::new(255, 255, 255)),
        ),
        Tile::CavernWall => (
            '#',
            TextInfo::default()
                .foreground_colour(Rgb24::new(15, 25, 0))
                .background_colour(Rgb24::new(60, 90, 0)),
        ),
        Tile::Door => (
            '+',
            TextInfo::default()
                .foreground_colour(Rgb24::new(255, 255, 255))
                .background_colour(Rgb24::new(0, 0, 127)),
        ),
        Tile::Floor => (
            '.',
            TextInfo::default()
                .foreground_colour(Rgb24::new(220, 220, 220))
                .background_colour(Rgb24::new(10, 10, 10)),
        ),
        Tile::Punch(direction) => {
            let ch = match direction {
                CardinalDirection::North => '↑',
                CardinalDirection::South => '↓',
                CardinalDirection::East => '→',
                CardinalDirection::West => '←',
            };
            (
                ch,
                TextInfo::default().bold().foreground_colour(Rgb24::new(255, 0, 0)),
            )
        }
        Tile::Larvae => {
            (
            'l',
            TextInfo::default()
                .bold()
                .foreground_colour(colours::BRIGHT_GREEN),
        )}
        Tile::Queen => (
            'Q',
            TextInfo::default()
                .bold()
                .foreground_colour(colours::BRIGHT_MAGENTA),
        ),
        Tile::Stairs => (
            '<',
            TextInfo::default()
                .bold()
                .foreground_colour(colours::BRIGHT_YELLOW),
        ),
        Tile::Exit => (
            'Ω',
            TextInfo::default()
                .bold()
                .foreground_colour(colours::BRIGHT_YELLOW),
        ),
        Tile::Bullet => (
            '•',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(150, 200, 50)),
        ),
        Tile::HealthPickup => (
            '♥',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(255, 30, 30)),
        ),
        Tile::AmmoPickup => (
            '‼',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(150, 200, 50)),
        ),
        Tile::RailGunShotHorizontal => (
            '═',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(0, 255, 255)),
        ),
        Tile::RailGunShotVertical => (
            '║',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(0, 255, 255)),
        ),
        Tile::RailGunAmmoPickup => (
            'ɸ',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(0, 255, 255)),
        ),
        Tile::KevlarPickup => (
            '♦',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(255, 63, 0)),
        ),
    };

    if tile_info.damage_flash {
        text_info.foreground_colour = Some(Rgb24::new(255, 0, 0));
    } else if tile_info.wounded {
        text_info.foreground_colour = Some(Rgb24::new(127, 0, 0));
    }

    (ch, text_info)
}

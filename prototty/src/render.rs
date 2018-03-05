use meters::tile::Tile;
use meters::tile_info::TileInfo;
use prototty::*;
use prototty_common::*;
use direction::CardinalDirection;

pub fn render_when_non_visible(tile: Tile) -> bool {
    match tile {
        Tile::Player | Tile::Punch(_) | Tile::Larvae | Tile::Queen | Tile::Bullet => false,
        Tile::Wall | Tile::CavernWall | Tile::Door | Tile::Floor | Tile::Stairs | Tile::Exit => true,
    }
}

pub fn tile_text(tile_info: TileInfo) -> (char, TextInfo) {
    match tile_info.tile {
        Tile::Player => (
            '@',
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(0, 255, 255)),
        ),
        Tile::Wall => (
            '#',
            TextInfo::default()
                .foreground_colour(Rgb24::new(80, 80, 80))
                .background_colour(Rgb24::new(220, 220, 220)),
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
                .foreground_colour(Rgb24::new(32, 7, 0))
                .background_colour(Rgb24::new(184, 34, 3)),
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
                TextInfo::default().foreground_colour(Rgb24::new(0, 255, 255)),
            )
        }
        Tile::Larvae => (
            'l',
            TextInfo::default()
                .bold()
                .foreground_colour(colours::BRIGHT_GREEN),
        ),
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
                .foreground_colour(Rgb24::new(0, 255, 255)),
        ),
    }
}



use grid_2d::*;
use entity_store::EntityIdAllocator;
use prototypes;
use super::*;

pub fn populate(
    strings: &Vec<String>,
    config: TerrainConfig,
    id_allocator: &mut EntityIdAllocator,
    messages: &mut MessageQueues,
) {
    for (y, line) in strings.iter().enumerate() {
        for (x, ch) in line.chars().enumerate() {
            let coord = Coord::new(x as i32, y as i32);
            match ch {
                '#' => {
                    prototypes::wall(id_allocator.allocate(), coord, messages);
                    prototypes::floor(id_allocator.allocate(), coord, messages);
                }
                '.' => {
                    prototypes::floor(id_allocator.allocate(), coord, messages);
                }
                'l' => {
                    prototypes::larvae(id_allocator.allocate(), coord, messages);
                    prototypes::floor(id_allocator.allocate(), coord, messages);
                }
                '<' => {
                    if config.final_level {
                        prototypes::exit(id_allocator.allocate(), coord, messages);
                    } else {
                        prototypes::stairs(id_allocator.allocate(), coord, messages);
                    }
                }
                '@' => {
                    let id = id_allocator.allocate();
                    prototypes::player(id, coord, messages);
                    prototypes::floor(id_allocator.allocate(), coord, messages);
                }
                _ => panic!("unexpected character"),
            }
        }
    }
}

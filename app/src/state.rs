use entity_store::*;
use cgmath::*;
use prototypes;

pub struct State {
    entity_store: EntityStore,
    spatial_hash: SpatialHashTable,
    player_id: EntityId,
    changes: Vec<EntityChange>,
}

impl State {
    pub fn new() -> Self {

        let strings = vec![
            "##########",
            "#........#",
            "#....#...#",
            "#..@.#...#",
            "#....#...#",
            "#.####...#",
            "#........#",
            "#........#",
            "#........#",
            "##########",
        ];

        let mut entity_store = EntityStore::new();
        let mut spatial_hash = SpatialHashTable::new(strings[0].len() as u32, strings.len() as u32);
        let mut id_allocator = EntityIdAllocator::new();
        let mut changes = Vec::new();
        let mut player_id = None;

        for (y, line) in strings.iter().enumerate() {
            for (x, ch) in line.chars().enumerate() {
                let coord = vec2(x as i32, y as i32);
                match ch {
                    '#' => {
                        prototypes::wall(id_allocator.allocate(), coord, &mut changes);
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    '.' => {
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    '@' => {
                        let id = id_allocator.allocate();
                        player_id = Some(id);
                        prototypes::player(id, coord, &mut changes);
                        prototypes::floor(id_allocator.allocate(), coord, &mut changes);
                    }
                    _ => panic!("unexpected character"),
                }
            }
        }

        let player_id = player_id.expect("No player in level");

        for change in changes.drain(..) {
            spatial_hash.update(&entity_store, &change, 0);
            entity_store.commit(change);
        }

        Self {
            entity_store,
            spatial_hash,
            player_id,
            changes,
        }
    }
}

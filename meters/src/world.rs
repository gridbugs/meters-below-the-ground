use rand::Rng;
use entity_store::*;
use grid_2d::*;
use message_queues::*;
use terrain::*;
use goal::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub entity_store: EntityStore,
    pub spatial_hash: SpatialHashTable,
    pub entity_components: EntityComponentTable,
    pub id_allocator: EntityIdAllocator,
    pub count: u64,
    pub goal_state: Option<GoalState>,
}

impl World {
    pub fn new<R: Rng>(terrain: &TerrainInfo, messages: &mut MessageQueues, rng: &mut R) -> Self {
        let size = terrain.size();

        let mut id_allocator = EntityIdAllocator::new();

        let goal_state = terrain.populate(&mut id_allocator, messages, rng);

        let mut world = Self {
            entity_store: EntityStore::new(),
            spatial_hash: SpatialHashTable::new(size),
            entity_components: EntityComponentTable::new(),
            id_allocator,
            count: 1,
            goal_state,
        };

        for change in messages.changes.drain(..) {
            world.commit(change);
        }

        world
    }

    pub fn commit(&mut self, change: EntityChange) {
        self.spatial_hash
            .update(&self.entity_store, &change, self.count);
        self.entity_components.update(&change);
        self.entity_store.commit(change);
    }

    pub fn size(&self) -> Size {
        self.spatial_hash.size()
    }

    pub fn component_drain_insert(
        &mut self,
        source_id: EntityId,
        dest_id: EntityId,
    ) -> ComponentDrainInsert {
        self.entity_components
            .component_drain_insert(source_id, dest_id, &mut self.entity_store)
    }
}

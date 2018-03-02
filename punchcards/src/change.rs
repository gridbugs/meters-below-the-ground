use std::collections::HashSet;
use rand::Rng;
use entity_store::*;
use message_queues::*;
use external_event::ExternalEvent;
use card_state::CardState;
use world::World;
use policy;

#[derive(Clone, Debug)]
pub struct ChangeContext {
    ids_to_free: HashSet<EntityId>,
}

impl ChangeContext {
    pub fn new() -> Self {
        Self {
            ids_to_free: HashSet::new(),
        }
    }

    pub fn process<R: Rng>(
        &mut self,
        world: &mut World,
        card_state: &mut CardState,
        messages: &mut MessageQueues,
        swap_messages: &mut MessageQueuesSwap,
        rng: &mut R,
    ) -> Option<ExternalEvent> {
        loop {
            for id in messages.removed_entities.drain(..) {
                for component in world.entity_components.components(id) {
                    messages.changes.push(EntityChange::Remove(id, component));
                }
                self.ids_to_free.insert(id);
            }

            if messages.changes.is_empty() {
                break;
            }

            for change in swap_drain!(changes, messages, swap_messages) {
                if let &EntityChange::Insert(id, ..) = &change {
                    if self.ids_to_free.contains(&id) {
                        continue;
                    }
                }

                if !policy::check(
                    &change,
                    &world.entity_store,
                    &world.spatial_hash,
                    &mut world.id_allocator,
                    messages,
                ) {
                    continue;
                }

                world
                    .spatial_hash
                    .update(&world.entity_store, &change, world.count);
                world.entity_components.update(&change);
                world.entity_store.commit(change);
            }

            for (id, card) in messages.take_cards.drain(..) {
                card_state.deck.add_random(card, rng);
                messages.removed_entities.push(id);
            }
        };

        for id in self.ids_to_free.drain() {
            world.id_allocator.free(id);
        }

        if card_state.hand.is_empty() {
            Some(ExternalEvent::GameOver)
        } else {
            None
        }
    }
}

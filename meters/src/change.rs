use std::collections::HashSet;
use rand::Rng;
use entity_store::*;
use message_queues::*;
use event::*;
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
        messages: &mut MessageQueues,
        swap_messages: &mut MessageQueuesSwap,
        rng: &mut R,
    ) -> Option<Event> {
        world.count += 1;

        loop {
            for id in messages.removed_entities.drain(..) {
                for change in world.entity_components.remove_entity(id) {
                    messages.changes.push(change);
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
                    rng,
                ) {
                    continue;
                }

                world
                    .spatial_hash
                    .update(&world.entity_store, &change, world.count);
                world.entity_components.update(&change);
                world.entity_store.commit(change);
            }

            if world.entity_store.push_wave.is_empty() {
                let mut pushed = world.entity_store.pushed.iter().collect::<Vec<_>>();
                pushed.sort_by(|a, b| {
                    b.1.distance.cmp(&a.1.distance)
                });
                for (id, pushed) in pushed.drain(..) {
                    if let Some(&coord) = world.entity_store.coord.get(id) {
                        let dest = coord + pushed.direction.coord();
                        messages.change(insert::coord(*id, dest));
                        if pushed.range == 0 {
                            messages.change(remove::pushed(*id));
                        } else {
                            let mut pushed = *pushed;
                            pushed.range -= 1;
                            messages.change(insert::pushed(*id, pushed));
                        }
                        if let Some(mut tile) = world.entity_store.tile_info.get(id).cloned() {
                            tile.pushed = false;
                            messages.change(insert::tile_info(*id, tile));
                        }
                    }
                }
            }
        }

        for id in self.ids_to_free.drain() {
            world.id_allocator.free(id);
        }

        match messages.special.take() {
            Some(Special::Lose) => Some(Event::External(ExternalEvent::Lose)),
            Some(Special::Win) => Some(Event::External(ExternalEvent::Win)),
            Some(Special::Ascend) => {
                let status = if let Some(goal) = world.goal_state.as_ref() {
                    if goal.is_complete(&world.entity_store) {
                        AscendStatus::CompleteGoal
                    } else {
                        AscendStatus::IncompleteGoal
                    }
                } else {
                    AscendStatus::NoGoal
                };
                Some(Event::External(ExternalEvent::Ascend(status)))
            }
            Some(Special::Alert(alert)) => Some(Event::External(ExternalEvent::Alert(alert))),
            None => None,
        }
    }
}

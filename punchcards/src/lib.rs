extern crate append;
extern crate cgmath;
extern crate direction;
#[macro_use] extern crate enum_primitive;
#[macro_use] extern crate entity_store_helper;
extern crate serde;
#[macro_use] extern crate serde_derive;

pub mod entity_store {
    include_entity_store!("entity_store.rs");
}

mod policy;
mod prototypes;
mod reaction;

pub mod cardinal_direction_table;
pub mod card_state;
pub mod input;
pub mod tile;
pub mod tile_info;
pub mod state;
pub mod card;

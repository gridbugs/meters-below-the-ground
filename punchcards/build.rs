#[macro_use]
extern crate entity_store_code_gen;

fn main() {
    generate_entity_store!("spec.toml", "entity_store.rs");
}

extern crate punchcards;
extern crate prototty;
extern crate direction;

use std::time::Duration;
use direction::CardinalDirection;
use punchcards::state::*;
use punchcards::tile::Tile;
use prototty::*;
use prototty::Input as ProtottyInput;
use prototty::inputs as prototty_inputs;
use punchcards::input::Input as PunchcardsInput;

fn view_tile<C: ViewCell>(tile: Tile, cell: &mut C) {
    match tile {
        Tile::Player => {
            cell.set_bold(true);
            cell.set_character('@');
            cell.set_foreground_colour(colours::WHITE);
        }
        Tile::Wall => {
            cell.set_foreground_colour(colours::BLACK);
            cell.set_background_colour(colours::WHITE);
            cell.set_character('#');
        }
        Tile::Floor => {
            cell.set_foreground_colour(Rgb24::new(127, 127, 127));
            cell.set_character('.');
        }
    }
}

pub struct StateView;
impl View<State> for StateView {
    fn view<G: ViewGrid>(&self, state: &State, offset: Coord, depth: i32, grid: &mut G) {

        let entity_store = state.entity_store();

        for (id, tile_info) in entity_store.tile_info.iter() {
            if let Some(coord) = entity_store.coord.get(&id) {
                if let Some(cell) = grid.get_mut(offset + Coord::new(coord.x, coord.y), tile_info.depth + depth) {
                    view_tile(tile_info.tile, cell);
                }
            }
        }
    }
}

const INITIAL_INPUT_BUFFER_SIZE: usize = 16;

pub struct App {
    state: State,
    input_buffer: Vec<PunchcardsInput>,
}

impl App {
    pub fn new() -> Self {
        let state = State::new();
        let input_buffer = Vec::with_capacity(INITIAL_INPUT_BUFFER_SIZE);

        Self { state, input_buffer }
    }
}

pub struct AppView;
impl View<App> for AppView {
    fn view<G: ViewGrid>(&self, app: &App, offset: Coord, depth: i32, grid: &mut G) {
        StateView.view(&app.state, offset, depth, grid);
    }
}

pub enum ControlFlow {
    Quit,
}

enum InputType {
    Game(PunchcardsInput),
    ControlFlow(ControlFlow),
}

impl App {
    pub fn tick<I>(&mut self, inputs: I, period: Duration) -> Option<ControlFlow>
        where I: IntoIterator<Item=ProtottyInput>,
    {
        for input in inputs {
            let input_type = match input {
                ProtottyInput::Up => InputType::Game(PunchcardsInput::Move(CardinalDirection::North)),
                ProtottyInput::Down => InputType::Game(PunchcardsInput::Move(CardinalDirection::South)),
                ProtottyInput::Left => InputType::Game(PunchcardsInput::Move(CardinalDirection::West)),
                ProtottyInput::Right => InputType::Game(PunchcardsInput::Move(CardinalDirection::East)),
                prototty_inputs::ETX => InputType::ControlFlow(ControlFlow::Quit),
                _ => continue,
            };
            match input_type {
                InputType::Game(input) => self.input_buffer.push(input),
                InputType::ControlFlow(control_flow) => {
                    return Some(control_flow);
                }
            }
        }

        self.state.tick(self.input_buffer.drain(..), period);

        None
    }
}

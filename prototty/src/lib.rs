extern crate punchcards;
extern crate prototty;
extern crate direction;

use std::fmt::Write;
use std::time::Duration;
use direction::CardinalDirection;
use punchcards::state::*;
use punchcards::tile::Tile;
use prototty::*;
use prototty::Input as ProtottyInput;
use prototty::inputs as prototty_inputs;
use punchcards::input::Input as PunchcardsInput;
use punchcards::card::Card;

const GAME_HEIGHT: u32 = 10;

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
        Tile::CardMove => {
            cell.set_foreground_colour(colours::YELLOW);
            cell.set_bold(true);
            cell.set_character('m');
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
        let string_buffer = String::with_capacity(INITIAL_STRING_BUFFER_SIZE);

        Self { state, input_buffer }
    }
}

const INITIAL_STRING_BUFFER_SIZE: usize = 16;

pub struct AppView {
    string_buffer: String,
}

impl AppView {
    pub fn new() -> Self {
        let string_buffer = String::with_capacity(INITIAL_STRING_BUFFER_SIZE);
        Self { string_buffer }
    }
}

fn write_card(card: Card, string: &mut String) {
    match card {
        Card::Move => write!(string, "Move").unwrap(),
    }
}

impl View<App> for AppView {
    fn view<G: ViewGrid>(&self, app: &App, offset: Coord, depth: i32, grid: &mut G) {

        let entity_store = app.state.entity_store();

        for (id, tile_info) in entity_store.tile_info.iter() {
            if let Some(coord) = entity_store.coord.get(&id) {
                if let Some(cell) = grid.get_mut(offset + Coord::new(coord.x, coord.y), tile_info.depth + depth) {
                    view_tile(tile_info.tile, cell);
                }
            }
        }

        let card_state = app.state.card_state();
        let mut string_buffer = String::new();
        write!(&mut string_buffer, "Deck Size: {}", card_state.deck().len());

        for (i, ch) in string_buffer.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(i as i32, 1 + GAME_HEIGHT as i32), depth) {
                cell.set_character(ch);
            }
        }
        string_buffer.clear();

        write!(&mut string_buffer, "Queue");
        for (i, ch) in string_buffer.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(i as i32, 3 + GAME_HEIGHT as i32), depth) {
                cell.set_character(ch);
            }
        }
        string_buffer.clear();

        for i in 0..card_state.queue_size() {
            write!(&mut string_buffer, "{}:" , i);
            if let Some(card) = card_state.queue().get(i) {
                write_card(*card, &mut string_buffer);
            } else {
                write!(&mut string_buffer, "-");
            }

            for (j, ch) in string_buffer.chars().enumerate() {
                if let Some(cell) = grid.get_mut(offset + Coord::new(j as i32, (4 + i as u32 + GAME_HEIGHT) as i32), depth) {
                    cell.set_character(ch);
                }
            }
            string_buffer.clear();
        }

        const QUEUE_WIDTH: i32 = 10;

        write!(&mut string_buffer, "Hand");
        for (i, ch) in string_buffer.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(QUEUE_WIDTH + i as i32, 3 + GAME_HEIGHT as i32), depth) {
                cell.set_character(ch);
            }
        }
        string_buffer.clear();

        if let Some(card) = card_state.hand().get(CardinalDirection::North) {
            write_card(*card, &mut string_buffer);
        } else {
            write!(&mut string_buffer, "-");
        }
        for (i, ch) in string_buffer.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(4 + QUEUE_WIDTH + i as i32, 4 + GAME_HEIGHT as i32), depth) {
                cell.set_character(ch);
            }
        }
        string_buffer.clear();

        write!(&mut string_buffer, "Hand");
        for (i, ch) in string_buffer.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(QUEUE_WIDTH + i as i32, 3 + GAME_HEIGHT as i32), depth) {
                cell.set_character(ch);
            }
        }
        string_buffer.clear();


        if let Some(card) = card_state.hand().get(CardinalDirection::East) {
            write_card(*card, &mut string_buffer);
        } else {
            write!(&mut string_buffer, "-");
        }
        for (i, ch) in string_buffer.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(9 + QUEUE_WIDTH + i as i32, 5 + GAME_HEIGHT as i32), depth) {
                cell.set_character(ch);
            }
        }
        string_buffer.clear();

        if let Some(card) = card_state.hand().get(CardinalDirection::South) {
            write_card(*card, &mut string_buffer);
        } else {
            write!(&mut string_buffer, "-");
        }
        for (i, ch) in string_buffer.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(4 + QUEUE_WIDTH + i as i32, 6 + GAME_HEIGHT as i32), depth) {
                cell.set_character(ch);
            }
        }
        string_buffer.clear();

        if let Some(card) = card_state.hand().get(CardinalDirection::West) {
            write_card(*card, &mut string_buffer);
        } else {
            write!(&mut string_buffer, "-");
        }
        for (i, ch) in string_buffer.chars().enumerate() {
            if let Some(cell) = grid.get_mut(offset + Coord::new(QUEUE_WIDTH + i as i32, 5 + GAME_HEIGHT as i32), depth) {
                cell.set_character(ch);
            }
        }
        string_buffer.clear();
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

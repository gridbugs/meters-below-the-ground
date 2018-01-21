extern crate punchcards;
extern crate prototty;
extern crate prototty_common;
extern crate direction;

use std::fmt::Write;
use std::time::Duration;
use direction::CardinalDirection;
use punchcards::state::*;
use punchcards::tile::Tile;
use prototty::*;
use prototty::Input as ProtottyInput;
use prototty::inputs as prototty_inputs;
use prototty_common::{StringView, RichStringView};
use punchcards::input::Input as PunchcardsInput;
use punchcards::card::Card;
use punchcards::state::Meta;

const GAME_OVER_MS: u64 = 1000;
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

#[derive(Debug, Clone, Copy)]
enum AppState {
    Game,
    GameOver,
}

pub struct App {
    app_state: AppState,
    state: State,
    input_buffer: Vec<PunchcardsInput>,
    game_over_duration: Duration,
}

impl App {
    pub fn new() -> Self {
        let state = State::new();
        let app_state = AppState::Game;
        let input_buffer = Vec::with_capacity(INITIAL_INPUT_BUFFER_SIZE);
        let game_over_duration = Duration::default();

        Self { state, app_state, input_buffer, game_over_duration }
    }
}

struct DeckView {
    scratch: String,
}

impl DeckView {
    fn new() -> Self {
        Self {
            scratch: String::new(),
        }
    }
}

struct QueueView {
    title_view: RichStringView,
    scratch: String,
}

impl QueueView {
    fn new() -> Self {
        let mut title_view = RichStringView::new();
        title_view.info.underline = true;
        Self {
            title_view,
            scratch: String::new(),
        }
    }
}

struct HandView {
    scratch: String,
    title_view: RichStringView,
}

impl HandView {
    fn new() -> Self {
        let mut title_view = RichStringView::new();
        title_view.info.underline = true;
        Self {
            title_view,
            scratch: String::new(),
        }
    }
}

struct HudView {
    deck_view: DeckView,
    queue_view: QueueView,
    hand_view: HandView,
}

impl HudView {
    fn new() -> Self {
        Self {
            deck_view: DeckView::new(),
            queue_view: QueueView::new(),
            hand_view: HandView::new(),
        }
    }
}

pub struct AppView {
    hud_view: HudView,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            hud_view: HudView::new(),
        }
    }
}

fn write_card(card: Card, string: &mut String) {
    match card {
        Card::Move => write!(string, "Move").unwrap(),
        Card::OtherMove => write!(string, "Move2").unwrap(),
    }
}

fn maybe_write_card(card: Option<Card>, string: &mut String) {
    if let Some(card)  = card {
        write_card(card, string);
    } else {
        write!(string, "-").unwrap();
    }
}

impl View<State> for DeckView {
    fn view<G: ViewGrid>(&mut self, state: &State, offset: Coord, depth: i32, grid: &mut G) {
        write!(&mut self.scratch, "Deck Size: {}", state.card_state().deck().len()).unwrap();
        StringView.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();
    }
}

impl View<State> for QueueView {
    fn view<G: ViewGrid>(&mut self, state: &State, offset: Coord, depth: i32, grid: &mut G) {

        write!(&mut self.scratch, "Next").unwrap();
        self.title_view.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();

        let card_state = state.card_state();
        let queue = card_state.queue();

        for i in 0..card_state.queue_size() {
            write!(&mut self.scratch, "{}: " , i).unwrap();
            maybe_write_card(queue.get(i).cloned(), &mut self.scratch);
            StringView.view(&self.scratch, offset + Coord::new(0, 1 + i as i32), depth, grid);
            self.scratch.clear();
        }
    }
}

impl View<State> for HandView {
    fn view<G: ViewGrid>(&mut self, state: &State, offset: Coord, depth: i32, grid: &mut G) {
        write!(&mut self.scratch, "Hand").unwrap();
        self.title_view.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();

        let hand = state.card_state().hand();

        maybe_write_card(hand.get(CardinalDirection::North).cloned(), &mut self.scratch);
        StringView.view(&self.scratch, offset + Coord::new(4, 1), depth, grid);
        self.scratch.clear();

        maybe_write_card(hand.get(CardinalDirection::East).cloned(), &mut self.scratch);
        StringView.view(&self.scratch, offset + Coord::new(9, 2), depth, grid);
        self.scratch.clear();

        maybe_write_card(hand.get(CardinalDirection::South).cloned(), &mut self.scratch);
        StringView.view(&self.scratch, offset + Coord::new(4, 3), depth, grid);
        self.scratch.clear();

        maybe_write_card(hand.get(CardinalDirection::West).cloned(), &mut self.scratch);
        StringView.view(&self.scratch, offset + Coord::new(0, 2), depth, grid);
        self.scratch.clear();
    }
}

impl View<State> for HudView {
    fn view<G: ViewGrid>(&mut self, state: &State, offset: Coord, depth: i32, grid: &mut G) {
        const QUEUE_WIDTH: i32 = 10;
        self.deck_view.view(state, offset, depth, grid);
        self.queue_view.view(state, offset + Coord::new(0, 2), depth, grid);
        self.hand_view.view(state, offset + Coord::new(QUEUE_WIDTH, 2), depth, grid);
    }
}

impl View<App> for AppView {
    fn view<G: ViewGrid>(&mut self, app: &App, offset: Coord, depth: i32, grid: &mut G) {
        match app.app_state {
            AppState::Game => {
                let entity_store = app.state.entity_store();

                for (id, tile_info) in entity_store.tile_info.iter() {
                    if let Some(coord) = entity_store.coord.get(&id) {
                        if let Some(cell) = grid.get_mut(offset + Coord::new(coord.x, coord.y), tile_info.depth + depth) {
                            view_tile(tile_info.tile, cell);
                        }
                    }
                }

                self.hud_view.view(&app.state, offset + Coord::new(0, GAME_HEIGHT as i32), depth, grid);
            }
            AppState::GameOver => {
                StringView.view(&"Game Over", offset, depth, grid);
            }
        }
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
        match self.app_state {
            AppState::Game => {
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

                if let Some(meta) = self.state.tick(self.input_buffer.drain(..), period) {
                    match meta {
                        Meta::GameOver => {
                            self.app_state = AppState::GameOver;
                            self.game_over_duration = Duration::from_millis(GAME_OVER_MS);
                        }
                    }
                }

                None
            }
            AppState::GameOver => {
                if let Some(remaining) = self.game_over_duration.checked_sub(period) {
                    self.game_over_duration = remaining;
                    None
                } else {
                    Some(ControlFlow::Quit)
                }
            }
        }
    }
}

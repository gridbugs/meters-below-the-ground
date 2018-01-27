extern crate punchcards;
extern crate prototty;
extern crate prototty_common;
extern crate direction;
extern crate rand;

use std::fmt::Write;
use std::time::Duration;
use direction::CardinalDirection;
use punchcards::state::*;
use punchcards::tile::Tile;
use prototty::*;
use prototty::Input as ProtottyInput;
use prototty::inputs as prototty_inputs;
use prototty_common::*;
use punchcards::input::Input as PunchcardsInput;
use punchcards::card::Card;
use punchcards::card_state::CardState;

use self::CardinalDirection::*;

const GAME_OVER_MS: u64 = 1000;
const GAME_HEIGHT: u32 = 10;
const GAME_WIDTH: u32 = 10;
const HAND_WIDTH: u32 = 12;
const HAND_HEIGHT: u32 = 8;
const DECK_WIDTH: u32 = 8;
const DECK_HEIGHT: u32 = 1;
const GAME_PADDING_BOTTOM: u32 = 1;
const GAME_PADDING_RIGHT: u32 = 1;

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
        Tile::Punch(direction) => {
            let ch = match direction {
                North | South => '|',
                East | West => '-',
            };
            cell.set_character(ch);
            cell.set_foreground_colour(colours::CYAN);
            cell.set_bold(false);
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
    pub fn new(seed: u32) -> Self {
        let state = State::new(seed);
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

struct HandView {
    scratch: String,
    selected_view: RichStringView,
}

impl HandView {
    fn new() -> Self {
        let mut selected_view = RichStringView::new();
        selected_view.info.foreground_colour = Some(colours::BLUE);
        selected_view.info.background_colour = Some(colours::WHITE);
        selected_view.info.bold = true;
        Self {
            scratch: String::new(),
            selected_view,
        }
    }
}

impl ViewSize<State> for HandView {
    fn size(&mut self, _state: &State) -> Size {
        Size::new(HAND_WIDTH, HAND_HEIGHT)
    }
}

pub struct AppView {
    deck_view: Decorated<DeckView, Border>,
    hand_view: Decorated<HandView, Border>,
}

impl AppView {
    pub fn new() -> Self {
        Self {
            deck_view: Decorated::new(DeckView::new(), Border::with_title("Deck")),
            hand_view: Decorated::new(HandView::new(), Border::with_title("Hand")),
        }
    }
}

fn write_card(card: Card, string: &mut String) {
    match card {
        Card::Move => write!(string, "Move").unwrap(),
        Card::Punch => write!(string, "Punch").unwrap(),
    }
}

fn maybe_write_card(card: Option<Card>, string: &mut String) {
    if let Some(card)  = card {
        write_card(card, string);
    } else {
        write!(string, "-").unwrap();
    }
}

impl View<CardState> for DeckView {
    fn view<G: ViewGrid>(&mut self, card_state: &CardState, offset: Coord, depth: i32, grid: &mut G) {
        write!(&mut self.scratch, "Size: {}", card_state.deck.num_cards()).unwrap();
        StringView.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();
    }
}

impl ViewSize<CardState> for DeckView {
    fn size(&mut self, _card_state: &CardState) -> Size {
        Size::new(DECK_WIDTH, DECK_HEIGHT)
    }
}

impl View<State> for HandView {
    fn view<G: ViewGrid>(&mut self, state: &State, offset: Coord, depth: i32, grid: &mut G) {

        let selected_index = if let &InputState::WaitingForDirection(index, _) = state.input_state() {
            Some(index)
        } else {
            None
        };

        for (i, maybe_card) in state.card_state().hand.iter().enumerate() {
            write!(&mut self.scratch, "{}: ", i + 1).unwrap();
            maybe_write_card(*maybe_card, &mut self.scratch);

            if Some(i) == selected_index {
                self.selected_view.view(&self.scratch, offset + Coord::new(0, i as i32), depth, grid);
            } else {
                StringView.view(&self.scratch, offset + Coord::new(0, i as i32), depth, grid);
            };

            self.scratch.clear();
        }
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

                self.deck_view.view(app.state.card_state(),
                    offset + Coord::new(0, GAME_HEIGHT as i32 + GAME_PADDING_BOTTOM as i32), depth, grid);

                self.hand_view.view(&app.state,
                    offset + Coord::new(GAME_WIDTH as i32 + GAME_PADDING_RIGHT as i32, 0), depth, grid);
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
                        ProtottyInput::Up => InputType::Game(PunchcardsInput::Direction(North)),
                        ProtottyInput::Down => InputType::Game(PunchcardsInput::Direction(South)),
                        ProtottyInput::Left => InputType::Game(PunchcardsInput::Direction(West)),
                        ProtottyInput::Right => InputType::Game(PunchcardsInput::Direction(East)),
                        ProtottyInput::Char('1') => InputType::Game(PunchcardsInput::SelectCard(0)),
                        ProtottyInput::Char('2') => InputType::Game(PunchcardsInput::SelectCard(1)),
                        ProtottyInput::Char('3') => InputType::Game(PunchcardsInput::SelectCard(2)),
                        ProtottyInput::Char('4') => InputType::Game(PunchcardsInput::SelectCard(3)),
                        ProtottyInput::Char('5') => InputType::Game(PunchcardsInput::SelectCard(4)),
                        ProtottyInput::Char('6') => InputType::Game(PunchcardsInput::SelectCard(5)),
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

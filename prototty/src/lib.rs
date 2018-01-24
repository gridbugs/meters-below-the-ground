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
use punchcards::state::Meta;

use self::CardinalDirection::*;

const GAME_OVER_MS: u64 = 1000;
const GAME_HEIGHT: u32 = 10;
const HAND_WIDTH: u32 = 12;
const HAND_HEIGHT: u32 = 3;
const QUEUE_WIDTH: u32 = 10;
const DECK_WIDTH: u32 = 10;
const DECK_HEIGHT: u32 = 1;
const HUD_SPACE: u32 = 1;

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

struct QueueView {
    scratch: String,
}

impl ViewSize<CardState> for QueueView {
    fn size(&mut self, card_state: &CardState) -> Size {
        Size::new(QUEUE_WIDTH, card_state.queue().max_size() as u32)
    }
}

impl QueueView {
    fn new() -> Self {
        let mut title_view = RichStringView::new();
        title_view.info.underline = true;
        Self {
            scratch: String::new(),
        }
    }
}

struct HandView {
    scratch: String,
    north_view: Decorated<StringView, Align>,
    east_view: Decorated<StringView, Align>,
    south_view: Decorated<StringView, Align>,
    west_view: Decorated<StringView, Align>,
}

impl HandView {
    fn new() -> Self {
        let mut title_view = RichStringView::new();
        title_view.info.underline = true;
        let size = Size::new(HAND_WIDTH, HAND_HEIGHT);

        use self::Alignment::*;
        let north_view = Decorated::new(StringView, Align::new(size, Centre, TopLeft));
        let south_view = Decorated::new(StringView, Align::new(size, Centre, BottomRight));
        let west_view = Decorated::new(StringView, Align::new(size, TopLeft, Centre));
        let east_view = Decorated::new(StringView, Align::new(size, BottomRight, Centre));
        Self {
            scratch: String::new(),
            north_view,
            east_view,
            south_view,
            west_view,
        }
    }
}

impl ViewSize<CardState> for HandView {
    fn size(&mut self, _card_state: &CardState) -> Size {
        Size::new(HAND_WIDTH, HAND_HEIGHT)
    }
}

struct HudView {
    deck_view: Decorated<DeckView, Border>,
    queue_view: Decorated<QueueView, Border>,
    hand_view: Decorated<HandView, Border>,
}

impl HudView {
    fn new() -> Self {
        Self {
            deck_view: Decorated::new(DeckView::new(), Border::with_title("Deck")),
            queue_view: Decorated::new(QueueView::new(), Border::with_title("Queue")),
            hand_view: Decorated::new(HandView::new(), Border::with_title("Hand")),
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
        write!(&mut self.scratch, "Size: {}", card_state.deck().num_cards()).unwrap();
        StringView.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();
    }
}

impl ViewSize<CardState> for DeckView {
    fn size(&mut self, _card_state: &CardState) -> Size {
        Size::new(DECK_WIDTH, DECK_HEIGHT)
    }
}

impl View<CardState> for QueueView {
    fn view<G: ViewGrid>(&mut self, card_state: &CardState, offset: Coord, depth: i32, grid: &mut G) {
        for (i, maybe_card) in card_state.queue().iter().enumerate() {
            write!(&mut self.scratch, "{}: " , i).unwrap();
            maybe_write_card(maybe_card.cloned(), &mut self.scratch);
            StringView.view(&self.scratch, offset + Coord::new(0, i as i32), depth, grid);
            self.scratch.clear();
        }
    }
}

impl View<CardState> for HandView {
    fn view<G: ViewGrid>(&mut self, card_state: &CardState, offset: Coord, depth: i32, grid: &mut G) {

        let hand = card_state.hand();

        maybe_write_card(hand.get(North).cloned(), &mut self.scratch);
        self.north_view.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();

        maybe_write_card(hand.get(East).cloned(), &mut self.scratch);
        self.east_view.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();

        maybe_write_card(hand.get(South).cloned(), &mut self.scratch);
        self.south_view.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();

        maybe_write_card(hand.get(West).cloned(), &mut self.scratch);
        self.west_view.view(&self.scratch, offset, depth, grid);
        self.scratch.clear();
    }
}

impl View<State> for HudView {
    fn view<G: ViewGrid>(&mut self, state: &State, offset: Coord, depth: i32, grid: &mut G) {

        let card_state = state.card_state();

        self.deck_view.view(card_state, offset, depth, grid);

        let deck_size = self.deck_view.size(card_state);
        self.queue_view.view(card_state, offset + Coord::new(0, deck_size.y() as i32), depth, grid);

        let hand_x = ::std::cmp::max(self.queue_view.size(card_state).x(), deck_size.x());
        self.hand_view.view(card_state, offset + Coord::new(hand_x as i32, 0), depth, grid);
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

                self.hud_view.view(&app.state, offset + Coord::new(0, GAME_HEIGHT as i32 + HUD_SPACE as i32), depth, grid);
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
                        ProtottyInput::Up => InputType::Game(PunchcardsInput::Move(North)),
                        ProtottyInput::Down => InputType::Game(PunchcardsInput::Move(South)),
                        ProtottyInput::Left => InputType::Game(PunchcardsInput::Move(West)),
                        ProtottyInput::Right => InputType::Game(PunchcardsInput::Move(East)),
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

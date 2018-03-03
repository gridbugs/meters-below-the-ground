extern crate direction;
#[macro_use]
extern crate itertools;
extern crate meters;
extern crate prototty;
extern crate prototty_common;
extern crate rand;

use std::time::Duration;
use rand::{Rng, SeedableRng, StdRng};
use direction::CardinalDirection;
use meters::state::*;
use meters::tile::Tile;
use meters::tile_info::TileInfo;
use prototty::*;
use prototty::Input as ProtottyInput;
use prototty::inputs as prototty_inputs;
use prototty_common::*;
use meters::input::Input as MetersInput;
use meters::ExternalEvent;

use self::CardinalDirection::*;

mod meter;
use self::meter::*;

mod goal;
use self::goal::*;

const SAVE_PERIOD_MS: u64 = 10000;
const SAVE_FILE: &'static str = "save";

const GAME_OVER_MS: u64 = 1000;
const GAME_WIDTH: u32 = 30;

const TITLE_WIDTH: u32 = 24;
const TITLE_HEIGHT: u32 = 6;

const GAME_TOP_PADDING: i32 = 2;

const METER_NAME_PADDING: usize = 9;
const METER_WIDTH: u32 = 11;

const OVERALL_PROGRESS_Y: i32 = 33;
const OVERALL_PROGRESS_METER_NAME_PADDING: usize = 21;
const OVERALL_PROGRESS_METER_WIDTH: u32 = 26;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frontend {
    Unix,
    Glutin,
    Wasm,
}

impl Frontend {
    fn supports_saving(self) -> bool {
        match self {
            Frontend::Wasm => false,
            _ => true,
        }
    }
}

fn view_tile<C: ViewCell>(tile_info: TileInfo, cell: &mut C) {
    match tile_info.tile {
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
        Tile::Punch(direction) => {
            let ch = match direction {
                North => '↑',
                South => '↓',
                East => '→',
                West => '←',
            };
            cell.set_character(ch);
            cell.set_foreground_colour(colours::CYAN);
            cell.set_bold(false);
        }
        Tile::Larvae => {
            cell.set_foreground_colour(colours::BRIGHT_GREEN);
            cell.set_bold(true);
            cell.set_character('l');
        }
        Tile::Stairs => {
            cell.set_foreground_colour(colours::WHITE);
            cell.set_bold(true);
            cell.set_character('<');
        }
        Tile::Exit => {
            cell.set_foreground_colour(colours::WHITE);
            cell.set_bold(true);
            cell.set_character('Ω');
        }
        Tile::Bullet => {
            cell.set_foreground_colour(colours::WHITE);
            cell.set_bold(true);
            cell.set_character('•');
        }
    }
}

const INITIAL_INPUT_BUFFER_SIZE: usize = 16;

#[derive(Debug, Clone, Copy)]
enum GameOverMessage {
    Lose,
    Win,
}

#[derive(Debug, Clone, Copy)]
enum AppState {
    Game,
    GameOver(GameOverMessage),
    MainMenu,
}

pub enum ControlFlow {
    Quit,
}

enum InputType {
    Game(MetersInput),
    ControlFlow(ControlFlow),
}

#[derive(Debug, Clone, Copy)]
enum MainMenuChoice {
    NewGame,
    Continue,
    SaveAndQuit,
    Save,
    Quit,
    ClearData,
}

struct TitleScreenView {
    title_view: RichStringView,
    main_menu_view: DefaultMenuInstanceView,
}

impl TitleScreenView {
    fn new() -> Self {
        Self {
            title_view: RichStringView::with_info(TextInfo::default().bold().underline()),
            main_menu_view: DefaultMenuInstanceView,
        }
    }
}

pub struct AppView {
    title_screen_view: Decorated<TitleScreenView, Align>,
    meter_view: MeterView,
    overall_progress_view: MeterView,
    goal_view: GoalView,
}

impl View<MenuInstance<MainMenuChoice>> for TitleScreenView {
    fn view<G: ViewGrid>(
        &mut self,
        menu: &MenuInstance<MainMenuChoice>,
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        self.title_view
            .view("Meters Below the Ground", offset, depth, grid);
        self.main_menu_view
            .view(menu, offset + Coord::new(0, 2), depth, grid);
    }
}
impl ViewSize<MenuInstance<MainMenuChoice>> for TitleScreenView {
    fn size(&mut self, _menu: &MenuInstance<MainMenuChoice>) -> Size {
        Size::new(TITLE_WIDTH, TITLE_HEIGHT)
    }
}

impl AppView {
    pub fn new(size: Size) -> Self {
        let align = Align::new(size, Alignment::Centre, Alignment::Centre);
        Self {
            title_screen_view: Decorated::new(TitleScreenView::new(), align),
            meter_view: MeterView::new(METER_NAME_PADDING, METER_WIDTH),
            overall_progress_view: MeterView::new(
                OVERALL_PROGRESS_METER_NAME_PADDING,
                OVERALL_PROGRESS_METER_WIDTH,
            ),
            goal_view: GoalView::new(),
        }
    }
    pub fn set_size(&mut self, size: Size) {
        self.title_screen_view.decorator.size = size;
    }
}

pub struct App<S: Storage> {
    main_menu: MenuInstance<MainMenuChoice>,
    app_state: AppState,
    state: State,
    in_progress: bool,
    input_buffer: Vec<MetersInput>,
    game_over_duration: Duration,
    rng: StdRng,
    storage: S,
    frontend: Frontend,
    save_remaining: Duration,
}

impl<S: Storage> View<App<S>> for AppView {
    fn view<G: ViewGrid>(&mut self, app: &App<S>, offset: Coord, depth: i32, grid: &mut G) {
        match app.app_state {
            AppState::MainMenu => {
                self.title_screen_view
                    .view(&app.main_menu, offset, depth, grid);
            }
            AppState::Game => {

                self.goal_view.view(&app.state.goal(), offset, depth, grid);

                let entity_store = app.state.entity_store();
                for (id, tile_info) in entity_store.tile_info.iter() {
                    if let Some(coord) = entity_store.coord.get(&id) {
                        if let Some(cell) = grid.get_mut(
                            offset + Coord::new(coord.x, coord.y + GAME_TOP_PADDING),
                            tile_info.depth + depth,
                        ) {
                            view_tile(*tile_info, cell);
                        }
                    }
                }

                let hud_offset = offset + Coord::new(GAME_WIDTH as i32, GAME_TOP_PADDING);
                for (y, info) in izip!(0..26, app.state.player_meter_info()) {
                    self.meter_view
                        .view(&info, hud_offset + Coord::new(0, y), depth, grid);
                }

                let overall_progress_offset = offset + Coord::new(0, OVERALL_PROGRESS_Y);
                const OVERALL_PROGRESS_TITLE: &'static str = "Metres below the ground";
                let overall_progress_meter = app.state.overall_progress_meter();
                self.overall_progress_view.view(
                    &(OVERALL_PROGRESS_TITLE, overall_progress_meter),
                    overall_progress_offset,
                    depth,
                    grid,
                );
            }
            AppState::GameOver(message) => {
                match message {
                    GameOverMessage::Lose => {
                        StringView.view(&"You Died", offset, depth, grid);
                    }
                    GameOverMessage::Win => {
                        StringView.view(&"You Win!", offset, depth, grid);
                    }
                }
            }
        }
    }
}

fn make_main_menu(in_progress: bool, frontend: Frontend) -> MenuInstance<MainMenuChoice> {
    let menu_items = if in_progress {
        vec![
            Some(("Continue", MainMenuChoice::Continue)),
            if frontend.supports_saving() {
                Some(("Save and Quit", MainMenuChoice::SaveAndQuit))
            } else {
                Some(("Save", MainMenuChoice::Save))
            },
            Some(("New Game", MainMenuChoice::NewGame)),
            Some(("Clear Data", MainMenuChoice::ClearData)),
        ].into_iter()
            .filter_map(|x| x)
            .collect()
    } else {
        vec![
            ("New Game", MainMenuChoice::NewGame),
            ("Quit", MainMenuChoice::Quit),
        ]
    };
    let main_menu = Menu::smallest(menu_items);
    MenuInstance::new(main_menu).unwrap()
}

impl<S: Storage> App<S> {
    pub fn new(frontend: Frontend, storage: S, seed: usize) -> Self {
        let mut rng = StdRng::from_seed(&[seed]);

        let existing_state: Option<SaveState> = storage.load(SAVE_FILE).ok();

        let (in_progress, state) = if let Some(state) = existing_state {
            (true, State::from(state))
        } else {
            (false, State::new(rng.gen()))
        };

        let main_menu = make_main_menu(in_progress, frontend);

        let app_state = AppState::MainMenu;
        let input_buffer = Vec::with_capacity(INITIAL_INPUT_BUFFER_SIZE);
        let game_over_duration = Duration::default();

        let save_remaining = Duration::from_millis(SAVE_PERIOD_MS);

        Self {
            main_menu,
            state,
            app_state,
            in_progress,
            input_buffer,
            game_over_duration,
            storage,
            rng,
            frontend,
            save_remaining,
        }
    }

    pub fn store(&mut self) {
        if self.in_progress {
            self.storage
                .store(SAVE_FILE, &self.state.save(self.rng.gen()))
                .expect("Failed to save");
        } else {
            match self.storage.remove_raw(SAVE_FILE) {
                Err(LoadError::IoError) => eprintln!("Failed to delete game data"),
                _ => (),
            }
        }
    }

    pub fn tick<I>(&mut self, inputs: I, period: Duration) -> Option<ControlFlow>
    where
        I: IntoIterator<Item = ProtottyInput>,
    {
        if period < self.save_remaining {
            self.save_remaining -= period;
        } else {
            self.save_remaining = Duration::from_millis(SAVE_PERIOD_MS);
            self.store();
        }

        match self.app_state {
            AppState::MainMenu => {
                if let Some(menu_output) = self.main_menu.tick(inputs) {
                    match menu_output {
                        MenuOutput::Quit => Some(ControlFlow::Quit),
                        MenuOutput::Cancel => {
                            if self.in_progress {
                                self.app_state = AppState::Game;
                            }
                            None
                        }
                        MenuOutput::Finalise(selection) => match selection {
                            MainMenuChoice::Quit => Some(ControlFlow::Quit),
                            MainMenuChoice::Save => {
                                self.store();
                                None
                            }
                            MainMenuChoice::SaveAndQuit => {
                                self.store();
                                Some(ControlFlow::Quit)
                            }
                            MainMenuChoice::Continue => {
                                self.app_state = AppState::Game;
                                self.in_progress = true;
                                None
                            }
                            MainMenuChoice::NewGame => {
                                self.state = State::new(self.rng.gen());
                                self.app_state = AppState::Game;
                                self.in_progress = true;
                                self.main_menu = make_main_menu(true, self.frontend);
                                self.store();
                                None
                            }
                            MainMenuChoice::ClearData => {
                                self.state = State::new(self.rng.gen());
                                self.in_progress = false;
                                self.main_menu = make_main_menu(false, self.frontend);
                                self.store();
                                None
                            }
                        },
                    }
                } else {
                    None
                }
            }
            AppState::Game => {
                for input in inputs {
                    let input_type = match input {
                        ProtottyInput::Up => InputType::Game(MetersInput::Direction(North)),
                        ProtottyInput::Down => InputType::Game(MetersInput::Direction(South)),
                        ProtottyInput::Left => InputType::Game(MetersInput::Direction(West)),
                        ProtottyInput::Right => InputType::Game(MetersInput::Direction(East)),
                        ProtottyInput::Char(' ') => InputType::Game(MetersInput::Wait),
                        ProtottyInput::Char(ch @ 'a'...'z') => {
                            InputType::Game(MetersInput::MeterSelect(ch))
                        }
                        prototty_inputs::ETX => InputType::ControlFlow(ControlFlow::Quit),
                        prototty_inputs::ESCAPE => {
                            if self.state.selected_meter_type().is_some() {
                                InputType::Game(MetersInput::MeterDeselect)
                            } else {
                                self.app_state = AppState::MainMenu;
                                break;
                            }
                        }
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
                        ExternalEvent::Lose => {
                            self.app_state = AppState::GameOver(GameOverMessage::Lose);
                            self.game_over_duration = Duration::from_millis(GAME_OVER_MS);
                        }
                        ExternalEvent::Win => {
                            self.app_state = AppState::GameOver(GameOverMessage::Win);
                            self.game_over_duration = Duration::from_millis(GAME_OVER_MS);
                        }
                    }
                }

                None
            }
            AppState::GameOver(_) => {
                if let Some(remaining) = self.game_over_duration.checked_sub(period) {
                    self.game_over_duration = remaining;
                } else {
                    self.in_progress = false;
                    self.main_menu = make_main_menu(false, self.frontend);
                    self.app_state = AppState::MainMenu;
                    self.state = State::new(self.rng.gen());
                }
                None
            }
        }
    }
}

extern crate direction;
#[macro_use]
extern crate itertools;
extern crate meters;
extern crate prototty_common;
extern crate prototty_input;
extern crate prototty_render;
extern crate prototty_storage;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use direction::CardinalDirection;
use meters::alert::*;
use meters::input::ActiveMeterIdentifier;
use meters::input::Input as MetersInput;
use meters::meter::*;
use meters::state::*;
use meters::tile_info::TileInfo;
use meters::*;
use prototty_common::*;
use prototty_input::inputs as prototty_inputs;
use prototty_input::Input as ProtottyInput;
use prototty_render::*;
use prototty_storage::*;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::BTreeSet;
use std::time::Duration;

use self::CardinalDirection::*;

mod meter;
use self::meter::*;

mod goal;
use self::goal::*;

mod glossary;
use self::glossary::*;

mod render;

const SAVE_PERIOD_MS: u64 = 10000;
const SAVE_FILE: &'static str = "save";

const GAME_OVER_MS: u64 = 4000;
const GAME_WIDTH: u32 = 29;

const GAME_TOP_PADDING: i32 = 5;

const METER_NAME_PADDING: usize = 9;
const METER_WIDTH: usize = 9;

const OVERALL_PROGRESS_Y: i32 = 41;
const OVERALL_PROGRESS_METER_NAME_PADDING: usize = 21;
const OVERALL_PROGRESS_METER_WIDTH: usize = 29;

const ACTIVE_METER_Y: i32 = 5;
const NUM_ACTIVE_METERS: i32 = 10;
const NUM_PASSIVE_METERS: i32 = 10;

const GOAL_TEXT_Y: i32 = 1;
const GOAL_METER_BOTTOM_Y: i32 = 29;

const GLOSSARY_TOP_Y: i32 = 36;

const MESSAGE_TOP_Y: i32 = 3;
const HELP_TOP_Y: i32 = 43;

const HELP: &'static str = "Move:↑←↑→  Wait:SPACE  Ability:0-9  Menu:ESC";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FullSaveState {
    game: SaveState,
    between_level_menu: Option<MenuInstance<BetweenLevelChoice>>,
    game_state: GameState,
}

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

fn colour_cell(fg: Option<Rgb24>, bg: Option<Rgb24>, visible: bool) -> ViewCellInfo {
    let mut cell = ViewCellInfo::new();
    if visible {
        if let Some(fg) = fg {
            cell.set_foreground(fg);
        }
        if let Some(bg) = bg {
            cell.set_background(bg);
        }
    } else {
        let b = |c| c / 8;
        let f = |c| c / 2;
        if let Some(Rgb24 { red, green, blue }) = fg {
            cell.set_foreground(Rgb24 {
                red: f(red),
                green: f(green),
                blue: f(blue),
            });
        }
        if let Some(Rgb24 { red, green, blue }) = bg {
            cell.set_background(Rgb24 {
                red: b(red),
                green: b(green),
                blue: b(blue),
            });
        }
    }
    cell
}

fn view_tile(tile_info: TileInfo, visibility: Visibility) -> ViewCellInfo {
    let visible = match visibility {
        Visibility::Visible => true,
        Visibility::Remembered => false,
    };
    if !visible && !render::render_when_non_visible(tile_info.tile) {
        return Default::default();
    }
    let (ch, info) = render::tile_text(tile_info);
    let mut cell = colour_cell(info.foreground_colour, info.background_colour, visible);
    cell.set_character(ch);
    if info.bold {
        cell.set_bold(true);
    }
    cell
}

const INITIAL_INPUT_BUFFER_SIZE: usize = 16;

#[derive(Debug, Clone, Copy)]
enum GameOverMessage {
    Lose,
    Win,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum GameState {
    Level,
    UpgradeMenu,
    NonUpgradeMenu,
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

struct GameMenuView {
    title_view: RichStringView,
    main_menu_view: DefaultMenuInstanceView,
}

impl GameMenuView {
    fn new(info: TextInfo) -> Self {
        Self {
            title_view: RichStringView::with_info(info),
            main_menu_view: DefaultMenuInstanceView::new(),
        }
    }
}

pub struct AppView {
    title_screen_view: Decorated<GameMenuView, Align>,
    between_level_view: Decorated<GameMenuView, Align>,
    meter_view: MeterView,
    overall_progress_view: MeterView,
    goal_view: GoalView,
    glossary_view: GlossaryView,
    glossary: BTreeSet<TileInfo>,
}

impl<'a, T: Copy> View<(&'static str, &'a MenuInstance<T>)> for GameMenuView {
    fn view<G: ViewGrid>(
        &mut self,
        &(title, menu): &(&'static str, &MenuInstance<T>),
        offset: Coord,
        depth: i32,
        grid: &mut G,
    ) {
        self.title_view.view(title, offset, depth, grid);
        self.main_menu_view
            .view(menu, offset + Coord::new(0, 2), depth, grid);
    }
}
impl<'a, T: Copy> ViewSize<(&'static str, &'a MenuInstance<T>)> for GameMenuView {
    fn size(&mut self, &(title, menu): &(&'static str, &MenuInstance<T>)) -> Size {
        Size::new(
            title.chars().count() as u32,
            menu.menu().entries.len() as u32,
        )
    }
}

impl AppView {
    pub fn new(size: Size) -> Self {
        let align = Align::new(size, Alignment::Centre, Alignment::Centre);
        let info = TextInfo::default().bold().underline();
        Self {
            title_screen_view: Decorated::new(GameMenuView::new(info), align),
            between_level_view: Decorated::new(GameMenuView::new(Default::default()), align),
            meter_view: MeterView::new(METER_NAME_PADDING, METER_WIDTH),
            overall_progress_view: MeterView::new(
                OVERALL_PROGRESS_METER_NAME_PADDING,
                OVERALL_PROGRESS_METER_WIDTH,
            ),
            goal_view: GoalView::new(),
            glossary_view: GlossaryView::new(),
            glossary: BTreeSet::new(),
        }
    }
    pub fn set_size(&mut self, size: Size) {
        self.title_screen_view.decorator.size = size;
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum BetweenLevelChoice {
    Upgrade(MeterType),
    Continue,
}

pub struct App<S: Storage> {
    main_menu: MenuInstance<MainMenuChoice>,
    between_level_menu: Option<MenuInstance<BetweenLevelChoice>>,
    app_state: AppState,
    game_state: GameState,
    state: State,
    in_progress: bool,
    input_buffer: Vec<MetersInput>,
    game_over_duration: Duration,
    rng: StdRng,
    storage: S,
    frontend: Frontend,
    save_remaining: Duration,
    alert: Option<Alert>,
}

impl<S: Storage> View<App<S>> for AppView {
    fn view<G: ViewGrid>(&mut self, app: &App<S>, offset: Coord, depth: i32, grid: &mut G) {
        match app.app_state {
            AppState::MainMenu => {
                self.title_screen_view.view(
                    &("Meters Below the Ground", &app.main_menu),
                    offset,
                    depth,
                    grid,
                );
            }
            AppState::Game => match app.game_state {
                GameState::Level => {
                    self.goal_view.view(
                        &app.state.goal_info(),
                        offset + Coord::new(0, GOAL_TEXT_Y),
                        depth,
                        grid,
                    );

                    self.glossary.clear();
                    for (tiles, coord, visibility) in app.state.visible_cells() {
                        for tile_info in tiles {
                            grid.set_cell(
                                offset + Coord::new(coord.x, coord.y + GAME_TOP_PADDING),
                                tile_info.depth + depth,
                                {
                                    let cell = view_tile(*tile_info, visibility);
                                    if visibility == Visibility::Visible
                                        || render::render_when_non_visible(tile_info.tile)
                                    {
                                        let mut tile_info = *tile_info;
                                        if let Some(countdown) = tile_info.countdown.as_mut() {
                                            *countdown = ::std::cmp::min(*countdown, 2); // fuck
                                        }
                                        tile_info.pushed = false;
                                        self.glossary.insert(tile_info);
                                    }
                                    cell
                                },
                            );
                        }
                    }

                    let mut active_end = 0;
                    let active_meter_offset = offset
                        + Coord::new(GAME_WIDTH as i32 + 1, GAME_TOP_PADDING + ACTIVE_METER_Y);
                    for (y, info) in
                        izip!(0..NUM_ACTIVE_METERS, app.state.player_active_meter_info())
                    {
                        self.meter_view.view(
                            &info,
                            active_meter_offset + Coord::new(0, y),
                            depth,
                            grid,
                        );
                        active_end += 1;
                    }

                    let passive_meter_offset = offset
                        + Coord::new(
                            GAME_WIDTH as i32 + 1,
                            GAME_TOP_PADDING + ACTIVE_METER_Y + active_end,
                        );
                    for (y, info) in
                        izip!(0..NUM_PASSIVE_METERS, app.state.player_passive_meter_info())
                    {
                        self.meter_view.view(
                            &info,
                            passive_meter_offset + Coord::new(0, y),
                            depth,
                            grid,
                        );
                    }

                    let mut from_bottom = 0;
                    app.state.with_goal_meters(|meter_info| {
                        let y = GOAL_METER_BOTTOM_Y - from_bottom;
                        self.meter_view.view(
                            &meter_info,
                            Coord::new(GAME_WIDTH as i32 + 1, y),
                            depth,
                            grid,
                        );
                        from_bottom += 1;
                    });

                    let overall_progress_offset = offset + Coord::new(0, OVERALL_PROGRESS_Y);
                    const OVERALL_PROGRESS_TITLE: &'static str = "Metres Below the Ground";
                    let overall_progress_meter = app.state.overall_progress_meter();
                    self.overall_progress_view.view(
                        &(OVERALL_PROGRESS_TITLE, overall_progress_meter),
                        overall_progress_offset,
                        depth,
                        grid,
                    );

                    let glossary_offset = offset + Coord::new(0, GLOSSARY_TOP_Y);
                    self.glossary_view
                        .view(&self.glossary, glossary_offset, depth, grid);

                    if let Some(alert) = app.alert {
                        TextInfoStringView.view(
                            &alert_str(alert),
                            offset + Coord::new(0, MESSAGE_TOP_Y),
                            depth,
                            grid,
                        );
                    }
                    StringView.view(HELP, offset + Coord::new(0, HELP_TOP_Y), depth, grid);
                }
                GameState::UpgradeMenu => {
                    if let Some(menu) = app.between_level_menu.as_ref() {
                        self.between_level_view.view(
                            &("As you climb the stairs, you find (choose one):", menu),
                            offset,
                            depth,
                            grid,
                        );
                    }
                }
                GameState::NonUpgradeMenu => {
                    if let Some(menu) = app.between_level_menu.as_ref() {
                        self.between_level_view.view(
                            &("As you climb the stairs, you find...nothing.", menu),
                            offset,
                            depth,
                            grid,
                        );
                    }
                }
            },
            AppState::GameOver(message) => match message {
                GameOverMessage::Lose => {
                    Decorated::new(
                        TextInfoStringView,
                        Align::new(
                            self.title_screen_view.decorator.size,
                            Alignment::Centre,
                            Alignment::Centre,
                        ),
                    )
                    .view(
                        &(
                            TextInfo::default().bold().foreground_colour(colours::RED),
                            "You Died",
                        ),
                        offset,
                        depth,
                        grid,
                    );
                }
                GameOverMessage::Win => {
                    Decorated::new(
                        TextInfoStringView,
                        Align::new(
                            self.title_screen_view.decorator.size,
                            Alignment::Centre,
                            Alignment::Centre,
                        ),
                    )
                    .view(
                        &(
                            TextInfo::default().bold().foreground_colour(colours::GREEN),
                            "You Escaped",
                        ),
                        offset,
                        depth,
                        grid,
                    );
                }
            },
        }
    }
}

fn make_upgrade_menu(upgrades: Vec<MeterType>) -> MenuInstance<BetweenLevelChoice> {
    let items = upgrades
        .iter()
        .map(|&typ| (meter::meter_name(typ), BetweenLevelChoice::Upgrade(typ)))
        .collect::<Vec<_>>();

    let menu = Menu::smallest(items);
    MenuInstance::new(menu).unwrap()
}

fn make_non_upgrade_menu() -> MenuInstance<BetweenLevelChoice> {
    let items = vec![("Continue", BetweenLevelChoice::Continue)];
    let menu = Menu::smallest(items);
    MenuInstance::new(menu).unwrap()
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
        ]
        .into_iter()
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

fn alert_str(alert: Alert) -> (TextInfo, &'static str) {
    match alert {
        Alert::NoStamina => (
            TextInfo::default().bold().foreground_colour(colours::RED),
            "Out of Stamina!",
        ),
        Alert::NoAmmo => (
            TextInfo::default().bold().foreground_colour(colours::RED),
            "Out of Ammo!",
        ),
        Alert::NoMedkit => (
            TextInfo::default().bold().foreground_colour(colours::RED),
            "Out of Medkits!",
        ),
        Alert::NoBlink => (
            TextInfo::default().bold().foreground_colour(colours::RED),
            "Out of Blink!",
        ),
        Alert::NoSuchMeter => (
            TextInfo::default().bold().foreground_colour(colours::RED),
            "No such meter!",
        ),
        Alert::WalkIntoWall => (
            TextInfo::default().bold().foreground_colour(colours::RED),
            "That location is impassible.",
        ),
        Alert::BlinkIntoNonEmpty => (
            TextInfo::default().bold().foreground_colour(colours::RED),
            "Destination is impassible.",
        ),
        Alert::ArmourBlock => (
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(255, 63, 0)),
            "Your armour absorbs the damage.",
        ),
        Alert::RailgunWhichDirection => (
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(0, 255, 255)),
            "Select a direction to fire.",
        ),
        Alert::BlinkWhichDirection => (
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(127, 63, 255)),
            "Select a direction to blink.",
        ),
        Alert::BeaconActive => (
            TextInfo::default()
                .bold()
                .foreground_colour(Rgb24::new(255, 0, 0)),
            "EMERGENCY BEACON ACTIVE BEEP BEEP BEEP BEEP",
        ),
    }
}

impl<S: Storage> App<S> {
    pub fn new(frontend: Frontend, storage: S, seed: usize) -> Self {
        let mut rng = StdRng::seed_from_u64(seed as u64);

        let existing_state: Option<FullSaveState> = storage.load(SAVE_FILE).ok();

        let (in_progress, state, between_level_menu, game_state) =
            if let Some(state) = existing_state {
                (
                    true,
                    State::from(state.game),
                    state.between_level_menu,
                    state.game_state,
                )
            } else {
                (false, State::new(rng.gen()), None, GameState::Level)
            };

        let main_menu = make_main_menu(in_progress, frontend);

        let app_state = AppState::MainMenu;
        let input_buffer = Vec::with_capacity(INITIAL_INPUT_BUFFER_SIZE);
        let game_over_duration = Duration::default();

        let save_remaining = Duration::from_millis(SAVE_PERIOD_MS);

        Self {
            main_menu,
            between_level_menu,
            state,
            game_state,
            app_state,
            in_progress,
            input_buffer,
            game_over_duration,
            storage,
            rng,
            frontend,
            save_remaining,
            alert: None,
        }
    }

    pub fn store(&mut self) {
        if self.in_progress {
            let state = FullSaveState {
                game: self.state.save(self.rng.gen()),
                between_level_menu: self.between_level_menu.clone(),
                game_state: self.game_state,
            };
            self.storage
                .store(SAVE_FILE, &state)
                .expect("Failed to save");
        } else {
            match self.storage.remove_raw(SAVE_FILE) {
                Err(LoadError::IoError) => eprintln!("Failed to delete game data"),
                _ => (),
            }
        }
    }

    pub fn tick<I>(&mut self, inputs: I, period: Duration, view: &AppView) -> Option<ControlFlow>
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
                if let Some(menu_output) = self
                    .main_menu
                    .tick_with_mouse(inputs, &view.title_screen_view.view.main_menu_view)
                {
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

                                if self.frontend == Frontend::Glutin {
                                    println!("RNG Seed: {}", self.state.rng_seed());
                                }

                                self.app_state = AppState::Game;
                                self.game_state = GameState::Level;
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
                match self.game_state {
                    GameState::Level => {
                        for input in inputs {
                            let input_type = match input {
                                ProtottyInput::Up => InputType::Game(MetersInput::Direction(North)),
                                ProtottyInput::Down => {
                                    InputType::Game(MetersInput::Direction(South))
                                }
                                ProtottyInput::Left => {
                                    InputType::Game(MetersInput::Direction(West))
                                }
                                ProtottyInput::Right => {
                                    InputType::Game(MetersInput::Direction(East))
                                }
                                ProtottyInput::Char(' ') => InputType::Game(MetersInput::Wait),
                                ProtottyInput::Char(ch @ '0'...'9') => {
                                    let identifier = ActiveMeterIdentifier::from_char(ch);
                                    InputType::Game(MetersInput::ActiveMeterSelect(identifier))
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

                        if !self.input_buffer.is_empty() {
                            self.alert = None;
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
                                ExternalEvent::Ascend(status) => match status {
                                    AscendStatus::IncompleteGoal | AscendStatus::NoGoal => {
                                        self.game_state = GameState::NonUpgradeMenu;
                                        self.between_level_menu = Some(make_non_upgrade_menu());
                                    }
                                    AscendStatus::CompleteGoal => {
                                        let choices = self.state.upgrade_choices();
                                        if choices.is_empty() {
                                            self.game_state = GameState::NonUpgradeMenu;
                                            self.between_level_menu = Some(make_non_upgrade_menu());
                                        } else {
                                            self.game_state = GameState::UpgradeMenu;
                                            self.between_level_menu =
                                                Some(make_upgrade_menu(choices));
                                        }
                                    }
                                },
                                ExternalEvent::Alert(alert) => {
                                    self.alert = Some(alert);
                                }
                            }
                        }
                    }
                    GameState::UpgradeMenu => {
                        if let Some(menu) = self.between_level_menu.as_mut() {
                            if let Some(output) = menu.tick_with_mouse(
                                inputs,
                                &view.between_level_view.view.main_menu_view,
                            ) {
                                match output {
                                    MenuOutput::Quit => return Some(ControlFlow::Quit),
                                    MenuOutput::Cancel => {
                                        self.app_state = AppState::MainMenu;
                                    }
                                    MenuOutput::Finalise(selection) => {
                                        if let BetweenLevelChoice::Upgrade(upgrade) = selection {
                                            self.state.switch_levels_upgrade(upgrade);
                                            self.app_state = AppState::Game;
                                            self.game_state = GameState::Level;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    GameState::NonUpgradeMenu => {
                        if let Some(menu) = self.between_level_menu.as_mut() {
                            if let Some(output) = menu.tick(inputs) {
                                match output {
                                    MenuOutput::Quit => return Some(ControlFlow::Quit),
                                    MenuOutput::Cancel => {
                                        self.app_state = AppState::MainMenu;
                                    }
                                    MenuOutput::Finalise(_) => {
                                        self.state.switch_levels_no_upgrade();
                                        self.app_state = AppState::Game;
                                        self.game_state = GameState::Level;
                                    }
                                }
                            }
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

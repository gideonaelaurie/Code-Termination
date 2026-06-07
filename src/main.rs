use bevy::prelude::*;
use bevy::window::{WindowMode, MonitorSelection};

const PLAYER_SIZE: Vec2 = Vec2::new(96.0, 96.0);
const GROUND_SIZE: Vec2 = Vec2::new(2200.0, 120.0);
const PLAYER_SPEED: f32 = 420.0;
const JUMP_SPEED: f32 = 720.0;
const GRAVITY: f32 = 1800.0;
const GROUND_Y: f32 = -110.0;

const DASH_SPEED: f32 = 1400.0;
const DASH_DURATION: f32 = 0.18;
const DOUBLE_TAP_TIMEOUT: f32 = 0.22;
const SMASHDOWN_SPEED: f32 = -2000.0;

const GLITCH_DURATION: f32 = 0.15;
const GLITCH_COOLDOWN: f32 = 1.0;
const GLITCH_DISTANCE: f32 = 180.0;
const WALL_SIZE: Vec2 = Vec2::new(40.0, 300.0);

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

#[derive(Component)]
struct JumpState {
    jumps_remaining: u32,
    max_jumps: u32,
    is_smashing: bool,
}

impl Default for JumpState {
    fn default() -> Self {
        Self {
            jumps_remaining: 2,
            max_jumps: 2,
            is_smashing: false,
        }
    }
}

#[derive(Component)]
struct DashState {
    last_a_press: f32,
    last_d_press: f32,
    dash_timer: f32,
    dash_dir: f32,
    air_dash_used: bool,
}

impl Default for DashState {
    fn default() -> Self {
        Self {
            last_a_press: -10.0,
            last_d_press: -10.0,
            dash_timer: 0.0,
            dash_dir: 0.0,
            air_dash_used: false,
        }
    }
}

#[derive(Component)]
struct GlitchState {
    timer: f32,
    cooldown: f32,
    just_ended: bool,
}

impl Default for GlitchState {
    fn default() -> Self {
        Self {
            timer: 0.0,
            cooldown: 0.0,
            just_ended: false,
        }
    }
}

#[derive(Component)]
struct Wall;

#[derive(Component)]
struct GameHUD;

#[derive(Component)]
struct Spike;

#[derive(Component)]
struct RamState {
    current: u32,
    max: u32,
    invulnerability_timer: f32,
}

impl Default for RamState {
    fn default() -> Self {
        Self {
            current: 6,
            max: 6,
            invulnerability_timer: 0.0,
        }
    }
}

#[derive(Component)]
struct RamHUD;

#[derive(Component)]
struct TutorialHUD;

#[derive(Resource)]
struct TutorialState {
    visible: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self { visible: true }
    }
}

#[derive(Resource, Default)]
struct PendingGameLoad {
    should_load: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default, States)]
enum AppState {
    #[default]
    TitleScreen,
    Game,
    Settings,
}

#[derive(Component)]
struct TitleScreenUI;

#[derive(Component, Clone, Copy)]
enum TitleButtonAction {
    Play,
    NewGame,
    Quit,
}

#[derive(Component)]
struct SettingsMenuUI;

#[derive(Component, Clone, Copy)]
enum SettingsButtonAction {
    Resume,
    Title,
    TutorialDone,
}

#[derive(Resource, Default)]
struct MenuSelection {
    selected_index: usize,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.14)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "gidames".into(),
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .init_state::<AppState>()
        .insert_resource(TutorialState::default())
        .insert_resource(PendingGameLoad::default())
        .insert_resource(MenuSelection::default())
        .add_systems(Startup, setup)
        // Title screen
        .add_systems(OnEnter(AppState::TitleScreen), setup_title_screen)
        .add_systems(OnExit(AppState::TitleScreen), cleanup_title_screen)
        .add_systems(Update, title_button_system.run_if(in_state(AppState::TitleScreen)))
        // Game
        .add_systems(
            OnTransition {
                exited: AppState::TitleScreen,
                entered: AppState::Game,
            },
            (reset_player_system, setup_game_hud_system).chain(),
        )
        .add_systems(Update, toggle_settings_menu)
        .add_systems(Update, (
            (
                move_player,
                jump_player,
                apply_velocity,
                update_glitch,
                handle_damage,
                resolve_collisions,
            ).chain(),
            update_hud,
            auto_save_system,
        ).run_if(in_state(AppState::Game)))
        .add_systems(OnExit(AppState::Game), save_game_state)
        // Pause/Settings menu
        .add_systems(OnEnter(AppState::Settings), setup_settings_menu)
        .add_systems(OnExit(AppState::Settings), cleanup_settings_menu)
        .add_systems(Update, settings_button_system.run_if(in_state(AppState::Settings)))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    // Spawning Ground
    commands.spawn((
        Sprite::from_color(Color::srgb(0.16, 0.18, 0.24), GROUND_SIZE),
        Transform::from_xyz(0.0, -220.0, 0.0),
    ));

    // Spawning Left Wall
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.25, 0.28, 0.38), WALL_SIZE),
        Transform::from_xyz(-300.0, -10.0, 1.0),
    ));

    // Spawning Right Wall
    commands.spawn((
        Wall,
        Sprite::from_color(Color::srgb(0.25, 0.28, 0.38), WALL_SIZE),
        Transform::from_xyz(300.0, -10.0, 1.0),
    ));

    // Spawning Spikes (Hazard zones)
    // Left Spike
    commands.spawn((
        Spike,
        Sprite::from_color(Color::srgb(0.9, 0.1, 0.2), Vec2::new(48.0, 48.0)),
        Transform::from_xyz(-600.0, -136.0, 1.0),
    ));

    // Right Spike
    commands.spawn((
        Spike,
        Sprite::from_color(Color::srgb(0.9, 0.1, 0.2), Vec2::new(48.0, 48.0)),
        Transform::from_xyz(600.0, -136.0, 1.0),
    ));

    // Spawning Player
    commands.spawn((
        Player,
        Velocity::default(),
        JumpState::default(),
        DashState::default(),
        GlitchState::default(),
        RamState::default(),
        Sprite::from_color(Color::srgb(0.48, 0.86, 0.62), PLAYER_SIZE),
        Transform::from_xyz(0.0, GROUND_Y, 1.0),
    ));
}

fn save_game(x: f32, y: f32, ram: u32, tutorial_visible: bool) {
    let content = format!("{},{},{},{}", x, y, ram, tutorial_visible);
    if let Err(e) = std::fs::write("save_game.txt", content) {
        eprintln!("Failed to save game: {:?}", e);
    }
}

fn load_game() -> Option<(f32, f32, u32, bool)> {
    let content = std::fs::read_to_string("save_game.txt").ok()?;
    let parts: Vec<&str> = content.trim().split(',').collect();
    if parts.len() == 4 {
        let x = parts[0].parse::<f32>().ok()?;
        let y = parts[1].parse::<f32>().ok()?;
        let ram = parts[2].parse::<u32>().ok()?;
        let tutorial_visible = parts[3].parse::<bool>().ok()?;
        Some((x, y, ram, tutorial_visible))
    } else {
        None
    }
}

fn reset_player_system(
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut JumpState, &mut DashState, &mut GlitchState, &mut RamState), With<Player>>,
    mut tutorial_state: ResMut<TutorialState>,
    pending_load: Res<PendingGameLoad>,
) {
    let mut loaded = false;
    if pending_load.should_load {
        if let Some((x, y, ram, tut_visible)) = load_game() {
            tutorial_state.visible = tut_visible;
            for (mut transform, mut velocity, mut jump_state, mut dash_state, mut glitch_state, mut ram_state) in &mut player_query {
                transform.translation = Vec3::new(x, y, 1.0);
                velocity.0 = Vec2::ZERO;
                *jump_state = JumpState::default();
                *dash_state = DashState::default();
                *glitch_state = GlitchState::default();
                ram_state.current = ram.min(ram_state.max);
            }
            loaded = true;
        }
    }

    if !loaded {
        tutorial_state.visible = true;
        for (mut transform, mut velocity, mut jump_state, mut dash_state, mut glitch_state, mut ram_state) in &mut player_query {
            transform.translation = Vec3::new(0.0, GROUND_Y, 1.0);
            velocity.0 = Vec2::ZERO;
            *jump_state = JumpState::default();
            *dash_state = DashState::default();
            *glitch_state = GlitchState::default();
            *ram_state = RamState::default();
        }
        // Write the fresh new save immediately
        save_game(0.0, GROUND_Y, 6, true);
    }
}

fn setup_title_screen(
    mut commands: Commands,
    hud_query: Query<Entity, With<GameHUD>>,
    mut menu_selection: ResMut<MenuSelection>,
) {
    menu_selection.selected_index = 0;
    for entity in &hud_query {
        commands.entity(entity).despawn();
    }

    // Spawn the black background container overlay
    commands.spawn((
        TitleScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgb(0.0, 0.0, 0.0)),
    ))
    .with_children(|parent| {
        // Title Text
        parent.spawn((
            Text::new("Code-Termination"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(40.0)),
                ..default()
            },
        ));

        // Play Button (Continue)
        parent.spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            TitleButtonAction::Play,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("PLAY"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // New Game Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                margin: UiRect::bottom(Val::Px(15.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            TitleButtonAction::NewGame,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("NEW GAME"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });

        // Quit Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(240.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            TitleButtonAction::Quit,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("QUIT"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });
    });
}

fn cleanup_title_screen(mut commands: Commands, query: Query<Entity, With<TitleScreenUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn title_button_system(
    mut button_query: Query<
        (Entity, &Interaction, &TitleButtonAction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut app_exit_events: MessageWriter<AppExit>,
    mut pending_load: ResMut<PendingGameLoad>,
    mut menu_selection: ResMut<MenuSelection>,
    gamepads: Query<&Gamepad>,
    mut stick_neutral: Local<bool>,
) {
    let mut buttons = button_query.iter_mut().collect::<Vec<_>>();
    buttons.sort_by_key(|(_, _, action, _)| match action {
        TitleButtonAction::Play => 0,
        TitleButtonAction::NewGame => 1,
        TitleButtonAction::Quit => 2,
    });
    let total_buttons = buttons.len();

    let mut up = false;
    let mut down = false;
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            up = true;
        }
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            down = true;
        }
    }

    let mut stick_up = false;
    let mut stick_down = false;
    let mut any_active = false;
    for gamepad in &gamepads {
        if let Some(y) = gamepad.get(GamepadAxis::LeftStickY) {
            if y > 0.5 {
                any_active = true;
                if *stick_neutral {
                    stick_up = true;
                }
            } else if y < -0.5 {
                any_active = true;
                if *stick_neutral {
                    stick_down = true;
                }
            }
        }
    }
    if any_active {
        *stick_neutral = false;
    } else {
        *stick_neutral = true;
    }

    if total_buttons > 0 {
        if up || stick_up {
            menu_selection.selected_index = if menu_selection.selected_index == 0 {
                total_buttons - 1
            } else {
                menu_selection.selected_index - 1
            };
        }
        if down || stick_down {
            menu_selection.selected_index = (menu_selection.selected_index + 1) % total_buttons;
        }
    }

    for (index, (_, interaction, _, _)) in buttons.iter().enumerate() {
        if *interaction == &Interaction::Hovered {
            menu_selection.selected_index = index;
        }
    }

    let gp_select = gamepads.iter().any(|g| g.just_pressed(GamepadButton::East));
    let mut trigger_action = None;
    for (index, (_, interaction, action, _)) in buttons.iter().enumerate() {
        if *interaction == &Interaction::Pressed || (gp_select && index == menu_selection.selected_index) {
            trigger_action = Some(*action);
        }
    }

    for (index, (_, interaction, _, mut bg_color)) in buttons.into_iter().enumerate() {
        if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
        } else if index == menu_selection.selected_index {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }

    if let Some(action) = trigger_action {
        match action {
            TitleButtonAction::Play => {
                pending_load.should_load = true;
                next_state.set(AppState::Game);
            }
            TitleButtonAction::NewGame => {
                pending_load.should_load = false;
                next_state.set(AppState::Game);
            }
            TitleButtonAction::Quit => {
                app_exit_events.write(AppExit::Success);
            }
        }
    }
}

fn auto_save_system(
    time: Res<Time>,
    mut timer: Local<f32>,
    player_query: Query<(&Transform, &RamState), With<Player>>,
    tutorial_state: Res<TutorialState>,
) {
    *timer += time.delta_secs();
    if *timer >= 2.0 {
        *timer = 0.0;
        if let Ok((transform, ram_state)) = player_query.single() {
            save_game(
                transform.translation.x,
                transform.translation.y,
                ram_state.current,
                tutorial_state.visible,
            );
        }
    }
}

fn save_game_state(
    player_query: Query<(&Transform, &RamState), With<Player>>,
    tutorial_state: Res<TutorialState>,
) {
    if let Ok((transform, ram_state)) = player_query.single() {
        save_game(
            transform.translation.x,
            transform.translation.y,
            ram_state.current,
            tutorial_state.visible,
        );
    }
}

fn setup_settings_menu(
    mut commands: Commands,
    tutorial_state: Res<TutorialState>,
    mut menu_selection: ResMut<MenuSelection>,
) {
    menu_selection.selected_index = 0;
    let show_tutorial_button = tutorial_state.visible;
    let card_height = if show_tutorial_button { 320.0 } else { 250.0 };

    commands.spawn((
        SettingsMenuUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.85)),
    ))
    .with_children(|parent| {
        // Modal panel card
        parent.spawn((
            Node {
                width: Val::Px(350.0),
                height: Val::Px(card_height),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(3.0)),
                padding: UiRect::all(Val::Px(25.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.09, 0.11, 0.16)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSE"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));

            // Resume button
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(45.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
                SettingsButtonAction::Resume,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Resume"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                ));
            });

            // Tutorial Done button
            if show_tutorial_button {
                parent.spawn((
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(45.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                    BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
                    SettingsButtonAction::TutorialDone,
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Tutorial Done"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.0, 1.0, 0.0)),
                    ));
                });
            }

            // Title Screen button
            parent.spawn((
                Button,
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(45.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
                BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
                SettingsButtonAction::Title,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("Title Screen"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                ));
            });
        });
    });
}

fn cleanup_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn settings_button_system(
    mut button_query: Query<
        (Entity, &Interaction, &SettingsButtonAction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut tutorial_state: ResMut<TutorialState>,
    tut_hud_query: Query<Entity, With<TutorialHUD>>,
    mut commands: Commands,
    mut menu_selection: ResMut<MenuSelection>,
    gamepads: Query<&Gamepad>,
    mut stick_neutral: Local<bool>,
) {
    let mut buttons = button_query.iter_mut().collect::<Vec<_>>();
    buttons.sort_by_key(|(_, _, action, _)| match action {
        SettingsButtonAction::Resume => 0,
        SettingsButtonAction::TutorialDone => 1,
        SettingsButtonAction::Title => 2,
    });
    let total_buttons = buttons.len();

    let mut up = false;
    let mut down = false;
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::DPadUp) {
            up = true;
        }
        if gamepad.just_pressed(GamepadButton::DPadDown) {
            down = true;
        }
    }

    let mut stick_up = false;
    let mut stick_down = false;
    let mut any_active = false;
    for gamepad in &gamepads {
        if let Some(y) = gamepad.get(GamepadAxis::LeftStickY) {
            if y > 0.5 {
                any_active = true;
                if *stick_neutral {
                    stick_up = true;
                }
            } else if y < -0.5 {
                any_active = true;
                if *stick_neutral {
                    stick_down = true;
                }
            }
        }
    }
    if any_active {
        *stick_neutral = false;
    } else {
        *stick_neutral = true;
    }

    if total_buttons > 0 {
        if up || stick_up {
            menu_selection.selected_index = if menu_selection.selected_index == 0 {
                total_buttons - 1
            } else {
                menu_selection.selected_index - 1
            };
        }
        if down || stick_down {
            menu_selection.selected_index = (menu_selection.selected_index + 1) % total_buttons;
        }
    }

    for (index, (_, interaction, _, _)) in buttons.iter().enumerate() {
        if *interaction == &Interaction::Hovered {
            menu_selection.selected_index = index;
        }
    }

    let gp_select = gamepads.iter().any(|g| g.just_pressed(GamepadButton::East));
    let mut trigger_action = None;
    for (index, (_, interaction, action, _)) in buttons.iter().enumerate() {
        if *interaction == &Interaction::Pressed || (gp_select && index == menu_selection.selected_index) {
            trigger_action = Some(*action);
        }
    }

    for (index, (_, interaction, _, mut bg_color)) in buttons.into_iter().enumerate() {
        if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
        } else if index == menu_selection.selected_index {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }

    if let Some(action) = trigger_action {
        match action {
            SettingsButtonAction::Resume => {
                next_state.set(AppState::Game);
            }
            SettingsButtonAction::Title => {
                next_state.set(AppState::TitleScreen);
            }
            SettingsButtonAction::TutorialDone => {
                tutorial_state.visible = false;
                for entity in &tut_hud_query {
                    commands.entity(entity).despawn();
                }
                next_state.set(AppState::Game);
            }
        }
    }
}

fn toggle_settings_menu(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let mut toggle = keyboard.just_pressed(KeyCode::Escape);
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::Start) {
            toggle = true;
        }
    }

    if toggle {
        match state.get() {
            AppState::Game => next_state.set(AppState::Settings),
            AppState::Settings => next_state.set(AppState::Game),
            _ => {}
        }
    }
}

fn update_glitch(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut query: Query<(&mut GlitchState, &mut Sprite, &mut Transform, &mut JumpState), With<Player>>,
) {
    let delta = time.delta_secs();

    for (mut glitch_state, mut sprite, mut transform, mut jump_state) in &mut query {
        glitch_state.just_ended = false;

        if glitch_state.timer > 0.0 {
            glitch_state.timer -= delta;
            if glitch_state.timer <= 0.0 {
                glitch_state.timer = 0.0;
                sprite.color = Color::srgb(0.48, 0.86, 0.62);
                glitch_state.just_ended = true;
            } else {
                let elapsed = time.elapsed_secs();
                if (elapsed * 30.0) as i32 % 2 == 0 {
                    sprite.color = Color::srgba(0.0, 1.0, 0.3, 0.3);
                } else {
                    sprite.color = Color::srgba(0.0, 0.5, 1.0, 0.6);
                }
            }
        }

        if glitch_state.cooldown > 0.0 {
            glitch_state.cooldown -= delta;
            if glitch_state.cooldown < 0.0 {
                glitch_state.cooldown = 0.0;
            }
        }

        let gp_glitch_pressed = gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::West)
                || g.just_pressed(GamepadButton::LeftTrigger)
                || g.just_pressed(GamepadButton::LeftTrigger2)
        });

        let shift_pressed = keyboard.just_pressed(KeyCode::ShiftLeft)
            || keyboard.just_pressed(KeyCode::ShiftRight)
            || keyboard.just_pressed(KeyCode::KeyF)
            || gp_glitch_pressed;

        if shift_pressed && glitch_state.cooldown == 0.0 && glitch_state.timer == 0.0 {
            glitch_state.timer = GLITCH_DURATION;
            glitch_state.cooldown = GLITCH_COOLDOWN;
            jump_state.is_smashing = false; // Cancel any active smashdown

            let mut dir = 0.0;
            if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
                dir -= 1.0;
            }
            if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
                dir += 1.0;
            }
            for gamepad in &gamepads {
                if gamepad.pressed(GamepadButton::DPadLeft) {
                    dir -= 1.0;
                }
                if gamepad.pressed(GamepadButton::DPadRight) {
                    dir += 1.0;
                }
                if let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX) {
                    if left_stick_x.abs() > 0.1 {
                        dir += left_stick_x;
                    }
                }
            }
            if dir == 0.0 {
                dir = 1.0;
            }
            let dir = dir.signum();

            transform.translation.x += dir * GLITCH_DISTANCE;
        }
    }
}

fn resolve_collisions(
    mut player_query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut JumpState,
            &mut DashState,
            &GlitchState,
        ),
        With<Player>,
    >,
    wall_query: Query<&Transform, (With<Wall>, Without<Player>)>,
) {
    for (mut player_trans, mut velocity, mut jump_state, mut dash_state, glitch_state) in &mut player_query {
        if glitch_state.timer > 0.0 {
            continue;
        }

        let player_x = player_trans.translation.x;
        let player_y = player_trans.translation.y;

        let player_left = player_x - PLAYER_SIZE.x / 2.0;
        let player_right = player_x + PLAYER_SIZE.x / 2.0;
        let player_top = player_y + PLAYER_SIZE.y / 2.0;
        let player_bottom = player_y - PLAYER_SIZE.y / 2.0;

        for wall_trans in &wall_query {
            let wall_x = wall_trans.translation.x;
            let wall_y = wall_trans.translation.y;

            let wall_left = wall_x - WALL_SIZE.x / 2.0;
            let wall_right = wall_x + WALL_SIZE.x / 2.0;
            let wall_top = wall_y + WALL_SIZE.y / 2.0;
            let wall_bottom = wall_y - WALL_SIZE.y / 2.0;

            if player_right > wall_left
                && player_left < wall_right
                && player_top > wall_bottom
                && player_bottom < wall_top
            {
                let overlap_x = if player_x < wall_x {
                    player_right - wall_left
                } else {
                    wall_right - player_left
                };

                let overlap_y = if player_y < wall_y {
                    player_top - wall_bottom
                } else {
                    wall_top - player_bottom
                };

                if overlap_x < overlap_y {
                    if player_x < wall_x {
                        player_trans.translation.x -= overlap_x;
                    } else {
                        player_trans.translation.x += overlap_x;
                    }
                } else {
                    if player_y < wall_y {
                        player_trans.translation.y -= overlap_y;
                        if velocity.0.y > 0.0 {
                            velocity.0.y = 0.0;
                        }
                    } else {
                        player_trans.translation.y += overlap_y;
                        if velocity.0.y < 0.0 {
                            velocity.0.y = 0.0;
                        }
                        jump_state.jumps_remaining = jump_state.max_jumps;
                        jump_state.is_smashing = false;
                        dash_state.air_dash_used = false;
                    }
                }
            }
        }
    }
}

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    gamepads: Query<&Gamepad>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut DashState,
            &mut Sprite,
            &JumpState,
            &GlitchState,
            &RamState,
        ),
        With<Player>,
    >,
) {
    let now = time.elapsed_secs();
    let delta = time.delta_secs();

    for (mut transform, mut dash_state, mut sprite, jump_state, glitch_state, ram_state) in &mut player_query {
        if jump_state.is_smashing {
            sprite.color = Color::srgb(1.0, 0.25, 0.0);
            dash_state.dash_timer = 0.0;
            continue;
        }

        let is_in_air = transform.translation.y > GROUND_Y;

        // Detect gamepad direction and button dash
        let mut gamepad_dir = 0.0;
        for gamepad in &gamepads {
            if gamepad.pressed(GamepadButton::DPadLeft) {
                gamepad_dir -= 1.0;
            }
            if gamepad.pressed(GamepadButton::DPadRight) {
                gamepad_dir += 1.0;
            }
            if let Some(left_stick_x) = gamepad.get(GamepadAxis::LeftStickX) {
                if left_stick_x.abs() > 0.1 {
                    gamepad_dir += left_stick_x;
                }
            }
        }
        gamepad_dir = gamepad_dir.clamp(-1.0, 1.0);

        let mut direction = 0.0;
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            direction -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            direction += 1.0;
        }
        direction += gamepad_dir;
        direction = direction.clamp(-1.0, 1.0);

        let gp_dash_pressed = gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::North)
                || g.just_pressed(GamepadButton::RightTrigger)
                || g.just_pressed(GamepadButton::RightTrigger2)
        });

        // Detect double tap for A or ArrowLeft (left)
        let mut should_dash = gp_dash_pressed;
        let mut dash_dir_input = if direction != 0.0 { direction.signum() } else { 1.0 };

        if keyboard.just_pressed(KeyCode::KeyA) || keyboard.just_pressed(KeyCode::ArrowLeft) {
            if now - dash_state.last_a_press < DOUBLE_TAP_TIMEOUT && dash_state.dash_timer <= 0.0 {
                should_dash = true;
                dash_dir_input = -1.0;
            }
            dash_state.last_a_press = now;
        }

        // Detect double tap for D or ArrowRight (right)
        if keyboard.just_pressed(KeyCode::KeyD) || keyboard.just_pressed(KeyCode::ArrowRight) {
            if now - dash_state.last_d_press < DOUBLE_TAP_TIMEOUT && dash_state.dash_timer <= 0.0 {
                should_dash = true;
                dash_dir_input = 1.0;
            }
            dash_state.last_d_press = now;
        }

        if should_dash && dash_state.dash_timer <= 0.0 {
            if !is_in_air || !dash_state.air_dash_used {
                dash_state.dash_timer = DASH_DURATION;
                dash_state.dash_dir = dash_dir_input;
                if is_in_air {
                    dash_state.air_dash_used = true;
                }
            }
        }

        // Apply movement
        let is_dashing = dash_state.dash_timer > 0.0;
        let is_glitching = glitch_state.timer > 0.0;

        if is_dashing {
            transform.translation.x += dash_state.dash_dir * DASH_SPEED * delta;
            dash_state.dash_timer -= delta;
            if dash_state.dash_timer < 0.0 {
                dash_state.dash_timer = 0.0;
            }
        } else {
            if direction != 0.0 {
                transform.translation.x += direction * PLAYER_SPEED * delta;
            }
        }

        // Apply sprite color based on status
        if is_glitching {
            // Color is handled dynamically by update_glitch system
        } else if ram_state.invulnerability_timer > 0.0 {
            // Flash red/orange and semi-transparent
            let elapsed = time.elapsed_secs();
            if (elapsed * 15.0) as i32 % 2 == 0 {
                sprite.color = Color::srgba(1.0, 0.2, 0.2, 0.5);
            } else {
                sprite.color = Color::srgba(0.48, 0.86, 0.62, 0.3);
            }
        } else if is_dashing {
            if dash_state.air_dash_used {
                sprite.color = Color::srgb(1.0, 0.3, 0.8);
            } else {
                sprite.color = Color::srgb(0.2, 0.9, 1.0);
            }
        } else {
            sprite.color = Color::srgb(0.48, 0.86, 0.62);
        }
    }
}

fn setup_game_hud(
    mut commands: Commands,
    tutorial_visible: bool,
) {
    commands.spawn((
        GameHUD,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            ..default()
        },
    ))
    .with_children(|parent| {
        if tutorial_visible {
            parent.spawn((
                TutorialHUD,
                Node {
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
            ))
            .with_children(|tut_parent| {
                // Tutorial Title
                tut_parent.spawn((
                    Text::new("=== SYSTEM TUTORIAL ==="),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 0.0)),
                    Node {
                        margin: UiRect::bottom(Val::Px(10.0)),
                        ..default()
                    },
                ));

                let tutorial_lines = [
                    "• MOVE: Use A/D or Arrow Keys",
                    "• JUMP: Space or Up Arrow (Double jump in mid-air)",
                    "• DASH: Double-tap A or D to dash horizontally",
                    "• SMASHDOWN: Press S or Down Arrow in mid-air to slam down",
                    "• GLITCH: Press SHIFT or F while moving to phase through walls and spikes",
                    "• WARNING: Touching red spikes or hitting a wall normally will deplete system RAM!",
                ];

                for line in tutorial_lines {
                    tut_parent.spawn((
                        Text::new(line),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.0, 1.0, 0.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(5.0)),
                            ..default()
                        },
                    ));
                }
            });
        }

        // System Memory Status (Health)
        parent.spawn((
            RamHUD,
            Text::new("SYSTEM MEMORY: 6GB / 6GB"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::top(Val::Px(15.0)),
                ..default()
            },
        ));
    });
}

fn setup_game_hud_system(
    commands: Commands,
    tutorial_state: Res<TutorialState>,
) {
    setup_game_hud(commands, tutorial_state.visible);
}

fn update_hud(
    player_query: Query<&RamState, With<Player>>,
    mut ram_hud_query: Query<&mut Text, With<RamHUD>>,
) {
    if let Ok(ram_state) = player_query.single() {
        if let Ok(mut text) = ram_hud_query.single_mut() {
            text.0 = format!("SYSTEM MEMORY: {}GB / {}GB", ram_state.current, ram_state.max);
        }
    }
}

fn handle_damage(
    time: Res<Time>,
    mut player_query: Query<
        (
            &mut Transform,
            &mut Velocity,
            &mut JumpState,
            &mut DashState,
            &mut GlitchState,
            &mut RamState,
        ),
        With<Player>,
    >,
    spike_query: Query<&Transform, (With<Spike>, Without<Player>)>,
    wall_query: Query<&Transform, (With<Wall>, Without<Player>)>,
) {
    let delta = time.delta_secs();

    if let Ok((
        mut player_trans,
        mut velocity,
        mut jump_state,
        mut dash_state,
        mut glitch_state,
        mut ram_state,
    )) = player_query.single_mut() {
        // Update invulnerability timer
        if ram_state.invulnerability_timer > 0.0 {
            ram_state.invulnerability_timer -= delta;
            if ram_state.invulnerability_timer < 0.0 {
                ram_state.invulnerability_timer = 0.0;
            }
        }

        let player_x = player_trans.translation.x;
        let player_y = player_trans.translation.y;
        let player_left = player_x - PLAYER_SIZE.x / 2.0;
        let player_right = player_x + PLAYER_SIZE.x / 2.0;
        let player_top = player_y + PLAYER_SIZE.y / 2.0;
        let player_bottom = player_y - PLAYER_SIZE.y / 2.0;

        let mut take_damage = false;

        // 1. Check spike collisions (hazardous red zones) - only if we are NOT currently glitching
        if glitch_state.timer == 0.0 {
            for spike_trans in &spike_query {
                let spike_x = spike_trans.translation.x;
                let spike_y = spike_trans.translation.y;
                let spike_size = Vec2::new(48.0, 48.0);
                let spike_left = spike_x - spike_size.x / 2.0;
                let spike_right = spike_x + spike_size.x / 2.0;
                let spike_top = spike_y + spike_size.y / 2.0;
                let spike_bottom = spike_y - spike_size.y / 2.0;

                if player_right > spike_left
                    && player_left < spike_right
                    && player_top > spike_bottom
                    && player_bottom < spike_top
                {
                    take_damage = true;
                    break;
                }
            }
        }

        // 2. Check wall glitch failures: only check if the glitch just ended this frame
        if !take_damage && glitch_state.just_ended {
            for wall_trans in &wall_query {
                let wall_x = wall_trans.translation.x;
                let wall_y = wall_trans.translation.y;
                let wall_left = wall_x - WALL_SIZE.x / 2.0;
                let wall_right = wall_x + WALL_SIZE.x / 2.0;
                let wall_top = wall_y + WALL_SIZE.y / 2.0;
                let wall_bottom = wall_y - WALL_SIZE.y / 2.0;

                if player_right > wall_left
                    && player_left < wall_right
                    && player_top > wall_bottom
                    && player_bottom < wall_top
                {
                    take_damage = true;
                    break;
                }
            }
        }

        // Apply damage if not invulnerable
        if take_damage && ram_state.invulnerability_timer == 0.0 {
            if ram_state.current >= 2 {
                ram_state.current -= 2;
            } else {
                ram_state.current = 0;
            }
            ram_state.invulnerability_timer = 1.0; // 1 second invulnerability

            // If crash, reset player
            if ram_state.current == 0 {
                player_trans.translation = Vec3::new(0.0, GROUND_Y, 1.0);
                velocity.0 = Vec2::ZERO;
                *jump_state = JumpState::default();
                *dash_state = DashState::default();
                *glitch_state = GlitchState::default();
                ram_state.current = ram_state.max; // Restore RAM
                ram_state.invulnerability_timer = 1.5; // Longer invuln on spawn
            }
        }
    }
}

fn jump_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut player_query: Query<(&Transform, &mut Velocity, &mut JumpState), With<Player>>,
) {
    let down_pressed = keyboard.just_pressed(KeyCode::KeyS)
        || keyboard.just_pressed(KeyCode::ArrowDown)
        || gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::DPadDown)
                || g.get(GamepadAxis::LeftStickY).unwrap_or(0.0) < -0.5
        });

    let up_pressed = keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::ArrowUp)
        || gamepads.iter().any(|g| {
            g.just_pressed(GamepadButton::South)
                || g.just_pressed(GamepadButton::East)
                || g.just_pressed(GamepadButton::DPadUp)
        });

    for (transform, mut velocity, mut jump_state) in &mut player_query {
        let is_in_air = transform.translation.y > GROUND_Y;

        if down_pressed && is_in_air {
            velocity.0.y = SMASHDOWN_SPEED;
            jump_state.is_smashing = true;
        } else if up_pressed && jump_state.jumps_remaining > 0 && !jump_state.is_smashing {
            velocity.0.y = JUMP_SPEED;
            jump_state.jumps_remaining -= 1;
        }
    }
}

fn apply_velocity(
    time: Res<Time>,
    mut player_query: Query<(&mut Transform, &mut Velocity, &mut JumpState, &mut DashState), With<Player>>,
) {
    for (mut transform, mut velocity, mut jump_state, mut dash_state) in &mut player_query {
        if dash_state.dash_timer > 0.0 {
            velocity.0.y = 0.0;
        } else {
            velocity.0.y -= GRAVITY * time.delta_secs();
        }
        
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        if transform.translation.y < GROUND_Y {
            transform.translation.y = GROUND_Y;
            velocity.0.y = 0.0;
            jump_state.jumps_remaining = jump_state.max_jumps;
            jump_state.is_smashing = false;
            dash_state.air_dash_used = false;
        }
    }
}

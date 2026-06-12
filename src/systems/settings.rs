use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

pub fn setup_settings_menu(
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

pub fn cleanup_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn settings_button_system(
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

pub fn toggle_settings_menu(
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

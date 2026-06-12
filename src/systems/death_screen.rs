use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

pub fn setup_death_screen(
    mut commands: Commands,
    hud_query: Query<Entity, With<GameHUD>>,
    mut menu_selection: ResMut<MenuSelection>,
) {
    menu_selection.selected_index = 0;
    for entity in &hud_query {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        DeathScreenUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.12, 0.02, 0.02, 0.85)),
    ))
    .with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(500.0),
                height: Val::Px(350.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(3.0)),
                padding: UiRect::all(Val::Px(25.0)),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.02, 0.02)),
            BorderColor::all(Color::srgb(1.0, 0.0, 0.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("▲ SYSTEM CRASH ▲"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.0, 0.0)),
            ));

            parent.spawn((
                Text::new("R.I.P. - CRITICAL MEMORY ERROR (0GB)"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Button,
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.05, 0.05)),
                BorderColor::all(Color::srgb(1.0, 0.0, 0.0)),
                DeathScreenButtonAction::Respawn,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("RESPAWN"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.0, 0.0)),
                ));
            });

            parent.spawn((
                Button,
                Node {
                    width: Val::Px(250.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.05, 0.05)),
                BorderColor::all(Color::srgb(1.0, 0.0, 0.0)),
                DeathScreenButtonAction::Title,
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("TITLE SCREEN"),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.0, 0.0)),
                ));
            });
        });
    });
}

pub fn cleanup_death_screen(mut commands: Commands, query: Query<Entity, With<DeathScreenUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn death_screen_button_system(
    mut button_query: Query<
        (Entity, &Interaction, &DeathScreenButtonAction, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    mut menu_selection: ResMut<MenuSelection>,
    gamepads: Query<&Gamepad>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut stick_neutral: Local<bool>,
    mut pending_load: ResMut<PendingGameLoad>,
) {
    let mut buttons = button_query.iter_mut().collect::<Vec<_>>();
    buttons.sort_by_key(|(_, _, action, _, _)| match action {
        DeathScreenButtonAction::Respawn => 0,
        DeathScreenButtonAction::Title => 1,
    });
    let total_buttons = buttons.len();

    let mut up = keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW);
    let mut down = keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS);
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

    for (index, (_, interaction, _, _, _)) in buttons.iter().enumerate() {
        if *interaction == &Interaction::Hovered {
            menu_selection.selected_index = index;
        }
    }

    let gp_select = gamepads.iter().any(|g| g.just_pressed(GamepadButton::East) || g.just_pressed(GamepadButton::South));
    let kb_select = keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space);
    let mut trigger_action = None;
    for (index, (_, interaction, action, _, _)) in buttons.iter().enumerate() {
        if *interaction == &Interaction::Pressed || ((gp_select || kb_select) && index == menu_selection.selected_index) {
            trigger_action = Some(*action);
        }
    }

    for (index, (_, interaction, _, mut bg_color, mut border_color)) in buttons.into_iter().enumerate() {
        if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.4, 0.1, 0.1));
            *border_color = BorderColor::all(Color::srgb(1.0, 0.5, 0.5));
        } else if index == menu_selection.selected_index {
            *bg_color = BackgroundColor(Color::srgb(0.25, 0.08, 0.08));
            *border_color = BorderColor::all(Color::srgb(1.0, 0.3, 0.3));
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.15, 0.05, 0.05));
            *border_color = BorderColor::all(Color::srgb(1.0, 0.0, 0.0));
        }
    }

    if let Some(action) = trigger_action {
        match action {
            DeathScreenButtonAction::Respawn => {
                pending_load.should_load = true;
                next_state.set(AppState::Game);
            }
            DeathScreenButtonAction::Title => {
                next_state.set(AppState::TitleScreen);
            }
        }
    }
}

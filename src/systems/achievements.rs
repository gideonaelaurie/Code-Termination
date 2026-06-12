use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

pub fn setup_achievements_screen(
    mut commands: Commands,
    achievements: Res<Achievements>,
) {
    commands.spawn((
        AchievementsMenuUI,
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
        // Title
        parent.spawn((
            Text::new("=== SYSTEM ACHIEVEMENTS ==="),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 1.0, 0.0)),
            Node {
                margin: UiRect::bottom(Val::Px(30.0)),
                ..default()
            },
        ));

        // List Container
        parent.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            margin: UiRect::bottom(Val::Px(40.0)),
            ..default()
        })
        .with_children(|list| {
            let tech_support_desc = format!(
                "Die ten times (Progress: {}/10).",
                achievements.death_count.min(10)
            );

            let list_data = [
                (
                    achievements.phase_shift,
                    "PHASE SHIFT".to_string(),
                    "Glitch through a wall or spike.".to_string(),
                ),
                (
                    achievements.turbo_charged,
                    "TURBO CHARGED".to_string(),
                    "Activate CPU Overclock.".to_string(),
                ),
                (
                    achievements.airborne,
                    "AIRBORNE".to_string(),
                    "Perform a double jump.".to_string(),
                ),
                (
                    achievements.system_crash,
                    "SYSTEM CRASH".to_string(),
                    "Deplete memory to 0GB.".to_string(),
                ),
                (
                    achievements.speed_daemon,
                    "SPEED DAEMON".to_string(),
                    "Perform a horizontal dash.".to_string(),
                ),
                (
                    achievements.better_call_tech_support,
                    "BETTER CALL TECH SUPPORT!".to_string(),
                    tech_support_desc,
                ),
            ];

            for (unlocked, name, desc) in &list_data {
                let status_icon = if *unlocked { "[X] " } else { "[ ] " };
                let full_text = format!("{}{}: {}", status_icon, name, desc);
                let color = if *unlocked {
                    Color::srgb(0.0, 1.0, 0.0)
                } else {
                    Color::srgb(0.4, 0.4, 0.4)
                };

                list.spawn((
                    Text::new(full_text),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(color),
                    Node {
                        margin: UiRect::bottom(Val::Px(15.0)),
                        ..default()
                    },
                ));
            }
        });

        // Back Button
        parent.spawn((
            Button,
            Node {
                width: Val::Px(200.0),
                height: Val::Px(50.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.15)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new("BACK"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));
        });
    });
}

pub fn cleanup_achievements_screen(mut commands: Commands, query: Query<Entity, With<AchievementsMenuUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn achievements_screen_system(
    mut button_query: Query<
        (Entity, &Interaction, &mut BackgroundColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
) {
    let mut back_pressed = keyboard.just_pressed(KeyCode::Escape) || keyboard.just_pressed(KeyCode::Backspace);
    for gamepad in &gamepads {
        if gamepad.just_pressed(GamepadButton::East) || gamepad.just_pressed(GamepadButton::Start) {
            back_pressed = true;
        }
    }

    for (_, interaction, mut bg_color) in &mut button_query {
        if *interaction == Interaction::Hovered {
            *bg_color = BackgroundColor(Color::srgb(0.2, 0.2, 0.25));
        } else if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.4));
            back_pressed = true;
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.12, 0.12, 0.15));
        }
    }

    if back_pressed {
        next_state.set(AppState::TitleScreen);
    }
}

pub fn update_achievement_toasts(
    time: Res<Time<Real>>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut AchievementToast, &mut BackgroundColor, &mut BorderColor)>,
) {
    for (entity, mut toast, mut bg, mut border) in &mut query {
        toast.timer -= time.delta_secs();
        if toast.timer <= 0.0 {
            commands.entity(entity).despawn();
        } else if toast.timer < 0.5 {
            let alpha = toast.timer / 0.5;
            bg.0 = Color::srgba(0.08, 0.09, 0.14, 0.95 * alpha);
            *border = BorderColor::all(Color::srgba(0.0, 1.0, 0.0, alpha));
        }
    }
}

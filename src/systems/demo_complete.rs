use bevy::prelude::*;
use crate::components::*;
use crate::resources::*;

pub fn setup_demo_complete(
    mut commands: Commands,
    hud_query: Query<Entity, With<GameHUD>>,
) {
    // Delete the save game when the player beats the demo
    let _ = std::fs::remove_file("save_game.txt");

    for entity in &hud_query {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        DemoCompleteUI,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.08, 0.05, 0.95)),
    ))
    .with_children(|parent| {
        parent.spawn((
            Node {
                width: Val::Px(550.0),
                height: Val::Px(350.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(3.0)),
                padding: UiRect::all(Val::Px(25.0)),
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.05, 0.03)),
            BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("▲ SYSTEM BREACH RESOLVED ▲"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 1.0, 0.0)),
            ));

            parent.spawn((
                Text::new("Thank you for playing the demo!"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 1.0, 0.5)),
            ));

            parent.spawn((
                Text::new("All memory sectors successfully stabilized."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.0, 0.8, 0.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Button,
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.05, 0.15, 0.05)),
                BorderColor::all(Color::srgb(0.0, 1.0, 0.0)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("RETURN TO TITLE SCREEN"),
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

pub fn cleanup_demo_complete(mut commands: Commands, query: Query<Entity, With<DemoCompleteUI>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn demo_complete_system(
    mut button_query: Query<
        (Entity, &Interaction, &mut BackgroundColor, &mut BorderColor),
        With<Button>,
    >,
    mut next_state: ResMut<NextState<AppState>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
) {
    let gp_select = gamepads.iter().any(|g| g.just_pressed(GamepadButton::East) || g.just_pressed(GamepadButton::South));
    let kb_select = keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::Space) || keyboard.just_pressed(KeyCode::Escape);
    let mut trigger_action = gp_select || kb_select;

    for (_, interaction, mut bg_color, mut border_color) in &mut button_query {
        if *interaction == Interaction::Hovered {
            *bg_color = BackgroundColor(Color::srgb(0.08, 0.25, 0.08));
            *border_color = BorderColor::all(Color::srgb(0.5, 1.0, 0.5));
        } else if *interaction == Interaction::Pressed {
            *bg_color = BackgroundColor(Color::srgb(0.1, 0.4, 0.1));
            *border_color = BorderColor::all(Color::srgb(1.0, 1.0, 1.0));
            trigger_action = true;
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.05, 0.15, 0.05));
            *border_color = BorderColor::all(Color::srgb(0.0, 1.0, 0.0));
        }
    }

    if trigger_action {
        next_state.set(AppState::TitleScreen);
    }
}

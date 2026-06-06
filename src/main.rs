use bevy::prelude::*;
use bevy::window::WindowResolution;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const PLAYER_SIZE: Vec2 = Vec2::new(96.0, 96.0);
const GROUND_SIZE: Vec2 = Vec2::new(2200.0, 120.0);
const PLAYER_SPEED: f32 = 420.0;
const JUMP_SPEED: f32 = 720.0;
const GRAVITY: f32 = 1800.0;
const GROUND_Y: f32 = -110.0;

#[derive(Component)]
struct Player;

#[derive(Component, Default)]
struct Velocity(Vec2);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.08, 0.09, 0.14)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "gidames".into(),
                resolution: WindowResolution::new(WINDOW_WIDTH, WINDOW_HEIGHT),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (move_player, jump_player, apply_velocity))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((
        Sprite::from_color(Color::srgb(0.16, 0.18, 0.24), GROUND_SIZE),
        Transform::from_xyz(0.0, -220.0, 0.0),
    ));

    commands.spawn((
        Player,
        Velocity::default(),
        Sprite::from_color(Color::srgb(0.48, 0.86, 0.62), PLAYER_SIZE),
        Transform::from_xyz(0.0, GROUND_Y, 1.0),
    ));
}

fn move_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let mut direction = 0.0;

    if keyboard.pressed(KeyCode::KeyA) {
        direction -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyD) {
        direction += 1.0;
    }

    if direction == 0.0 {
        return;
    }

    for mut transform in &mut player_query {
        transform.translation.x += direction * PLAYER_SPEED * time.delta_secs();
    }
}

fn jump_player(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&Transform, &mut Velocity), With<Player>>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    for (transform, mut velocity) in &mut player_query {
        if transform.translation.y <= GROUND_Y {
            velocity.0.y = JUMP_SPEED;
        }
    }
}

fn apply_velocity(time: Res<Time>, mut player_query: Query<(&mut Transform, &mut Velocity), With<Player>>) {
    for (mut transform, mut velocity) in &mut player_query {
        velocity.0.y -= GRAVITY * time.delta_secs();
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();

        if transform.translation.y < GROUND_Y {
            transform.translation.y = GROUND_Y;
            velocity.0.y = 0.0;
        }
    }
}

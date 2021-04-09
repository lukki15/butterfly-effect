use bevy::{prelude::*, render::pass::ClearColor};
mod consts {
    pub const MAX_BOARD_WIDTH: i32 = 31;
    pub const MAX_BOARD_HEIGHT: i32 = 16;
    pub const STATUS_HEIGHT: i32 = 2;
    pub const PIXELS: i32 = 16;
}

fn main() {
    let mut app = App::build();
    app.add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)));
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    app.add_startup_system(setup.system())
    .add_startup_stage("game_setup", SystemStage::single(spawn_rocket.system()))
    .add_system(rocket_movement.system())
    .add_system(rocket_direction_control.system())
    .run();
}

struct Rocket {
    direction: Direction,
}

struct Scoreboard {
    score: usize,
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

enum Collider {
    Solid,
    Door,
    Rocket,
}

fn setup(mut commands: Commands) {
    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
}

fn spawn_rocket(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = asset_server.load("icon.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            sprite: Sprite::new(Vec2::new(16.0, 16.0)),
            ..Default::default()
        })
        .insert(Rocket {
            direction: Direction::Down,
        });
}

fn rocket_movement(mut rocket_position: Query<(&Rocket, &mut Transform)>) {
        
    for (rocket, mut transform) in rocket_position.iter_mut() {
        match rocket.direction {
            Direction::Up => transform.translation.y += 2.,
            Direction::Down => transform.translation.y -= 2.,
            Direction::Right => transform.translation.x += 2.,
            Direction::Left => transform.translation.x -= 2.,
        }
        
    }
}

fn rocket_direction_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<&mut Rocket>,
) {

    if let Ok(mut rocket) = query.single_mut() {
        if keyboard_input.pressed(KeyCode::Left) {
            rocket.direction = Direction::Left;
        }
        if keyboard_input.pressed(KeyCode::Right) {
            rocket.direction = Direction::Right;
        }
        if keyboard_input.pressed(KeyCode::Down) {
            rocket.direction = Direction::Down
        }
        if keyboard_input.pressed(KeyCode::Up) {
            rocket.direction = Direction::Up;
        }
    }
}
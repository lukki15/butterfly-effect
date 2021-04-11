use bevy::core::FixedTimestep;
use bevy::prelude::*;
use bevy::render::pass::ClearColor;

const SCORE_BOARD_HEIGHT: u32 = 2;
const ARENA_HEIGHT: u32 = 16;
const ARENA_WIDTH: u32 = 24;

const SPRITE_HEIGHT: u32 = 32;
const SPRITE_WIDTH: u32 = 32;

const WINDOW_HEIGHT: u32 = (ARENA_HEIGHT + SCORE_BOARD_HEIGHT) * SPRITE_HEIGHT;
const WINDOW_WIDTH: u32 = ARENA_WIDTH * SPRITE_WIDTH;

const MAX_TURNS: u32 = 10;

#[derive(SystemLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum RocketMovement {
    Input,
    Movement,
    Reset,
    Target,
    Loading,
    Path,
}

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

struct Rocket {
    direction: Direction,
    turns_left: u32,
}

#[derive(Default)]
struct RocketPath(Vec<Position>, Vec<Entity>);

struct Wall {}
struct Target {}
struct TargetEvent();
struct FindPathEvent();

struct ResetEvent();

struct NextLevelEvent();
struct GameOverEvent();

#[derive(Default)]
struct LevelInfo {
    current_level: usize,
    counter_completion: u32,
}

#[derive(PartialEq, Copy, Clone)]
enum Direction {
    Left,
    Up,
    Right,
    Down,
    StandStill,
}

impl Direction {
    fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
            Self::StandStill => Self::StandStill,
        }
    }
}

fn spawn_wall(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
    wall_position: Position,
) -> Entity {
    let texture_handle = asset_server.load("LunarLander/Moon Tiles/MoonTile_square.png");
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Wall {})
        .insert(wall_position)
        .insert(Size::square(0.9))
        .id()
}

fn _spawn_debris(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
    wall_position: Position,
    index: usize,
) {
    let path = format!("LunarLander/Space Background/debris_{}.png", index % 6);
    let texture_handle = asset_server.load(&path[..]);
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Wall {})
        .insert(wall_position)
        .insert(Size::square(0.9));
}

fn spawn_target(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    wall_position: Position,
) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(1.0, 0.8, 0.0).into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Target {})
        .insert(wall_position)
        .insert(Size::square(0.9));
}

fn spawn_border(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
) {
    for y in 0..ARENA_HEIGHT as i32 {
        spawn_wall(commands, materials, asset_server, Position { x: 0, y });
        spawn_wall(
            commands,
            materials,
            asset_server,
            Position {
                x: ARENA_WIDTH as i32 - 1,
                y,
            },
        );
    }
    for x in 1..ARENA_WIDTH as i32 - 1 {
        spawn_wall(commands, materials, asset_server, Position { x, y: 0 });
        spawn_wall(
            commands,
            materials,
            asset_server,
            Position {
                x,
                y: ARENA_HEIGHT as i32 - 1,
            },
        );
    }
}

fn load_game_over(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    wall_query: Query<Entity, With<Wall>>,
    target_query: Query<Entity, With<Target>>,
    mut reader: EventReader<GameOverEvent>,
) {
    if reader.iter().next().is_some() {
        // unload all walls
        for wall in wall_query.iter() {
            commands.entity(wall).despawn();
        }

        for target in target_query.iter() {
            commands.entity(target).despawn();
        }

        let game_over_data = vec![
            "                     ".to_string(),
            "  WWW  WWW W   W WWW ".to_string(),
            "  W    W W WW WW W   ".to_string(),
            "  W WW WWW W W W WWW ".to_string(),
            "  W  W W W W   W W   ".to_string(),
            "  WWWW W W W   W WWW ".to_string(),
            "                     ".to_string(),
            "   WWW W  W WWW WWW  ".to_string(),
            "   W W W  W W   W W  ".to_string(),
            "   W W W  W WWW WWW  ".to_string(),
            "   W W W  W W   WW   ".to_string(),
            "   WWW  WW  WWW W W  ".to_string(),
            "                     ".to_string(),
            "                     ".to_string(),
        ];

        load_level_from_data(
            &mut commands,
            &mut materials,
            &asset_server,
            &game_over_data,
        );
    }
}

fn load_level(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
    current_level: usize,
) {
    let level_data_0: Vec<String> = vec![
        "WWWWWWWWWWW WWWWWWWWWT".to_string(),
        "WWWWWWWWWW   WWWWWWWW ".to_string(),
        "WWWWWWWWW             ".to_string(),
        "WWWWWWWWWW   WWWWWWWW ".to_string(),
        "WWWWWWWWWWW WWWWWWWWW ".to_string(),
        "WWWWWWWWWWW WWWWWWWW  ".to_string(),
        "                      ".to_string(),
        "  WWWWWWWWW WWWWWWWWWW".to_string(),
        " WWWWWWWWWW WWWWWWWWWW".to_string(),
        " WWWWWWWWW  WWWWWWWWWW".to_string(),
        "            WWWWWWWWWW".to_string(),
        " WWWWWWWWW  WWWWWWWWWW".to_string(),
        " WWWWWWWWWW WWWWWWWWWW".to_string(),
        "SWWWWWWWWWWWWWWWWWWWWW".to_string(),
    ];
    let level_data_1: Vec<String> = vec![
        "          WWWW       T".to_string(),
        "   WWWW   WWWW        ".to_string(),
        "   WWWW   WWWW        ".to_string(),
        "   WWWW   WWWW        ".to_string(),
        "   WWWW   WWWW        ".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW   WWWW   WWWWW".to_string(),
        "   WWWW          WWWWW".to_string(),
        "   WWWW          WWWWW".to_string(),
        "   WWWW          WWWWW".to_string(),
        "s WWWWW          WWWWW".to_string(),
    ];

    let level_data_2: Vec<String> = vec![
        "                      ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        " WWWWWWWWW W WWWWWWWW ".to_string(),
        "                      ".to_string(),
        " WWWWWWWWW T WWWWWWWW ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        " WWWWWWWWWW WWWWWWWWW ".to_string(),
        "S                     ".to_string(),
    ];

    let level_data_3: Vec<String> = vec![
        "          WW          ".to_string(),
        "          WW          ".to_string(),
        "         WTTW         ".to_string(),
        "         W  W         ".to_string(),
        "W    W   W  W   W    W".to_string(),
        " TW  W  W    W  W  WT ".to_string(),
        " W W W W  WW  W W W W ".to_string(),
        " W  WW W W  W W WW  W ".to_string(),
        "  W  WW  W  W  WW  W  ".to_string(),
        "   W W    WW    W W   ".to_string(),
        "    WW          WW    ".to_string(),
        "     WWWWW  WWWWW     ".to_string(),
        "        WW  WW        ".to_string(),
        "S                     ".to_string(),
    ];

    let you_won = vec![
        "                     T".to_string(),
        "     W W  WWW  W W    ".to_string(),
        "     W W  W W  W W    ".to_string(),
        "     WWW  W W  W W    ".to_string(),
        "      W   W W  W W    ".to_string(),
        "      W   WWW  WWW    ".to_string(),
        "                      ".to_string(),
        "                      ".to_string(),
        "    W   W WWW W   W   ".to_string(),
        "    W   W W W WW  W   ".to_string(),
        "    W   W W W W W W   ".to_string(),
        "    W W W W W W  WW   ".to_string(),
        "     W W  WWW W   W   ".to_string(),
        "                      ".to_string(),
    ];

    let levels = vec![&level_data_0, &level_data_1, &level_data_2, &level_data_3];

    let current_level_data = levels.get(current_level);
    let level_data: &Vec<String>;

    if let Some(data) = current_level_data {
        level_data = data;
    } else {
        level_data = &you_won;
    }

    load_level_from_data(commands, materials, asset_server, level_data);
}

fn load_level_from_data(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    asset_server: &Res<AssetServer>,
    level_data: &Vec<String>,
) {
    for (y, line_data) in level_data.iter().rev().enumerate() {
        for (x, c) in line_data.chars().enumerate() {
            let pos = Position {
                x: x as i32 + 1,
                y: y as i32 + 1,
            };
            if c == 'W' {
                spawn_wall(commands, materials, asset_server, pos);
            } else if c == 'T' {
                spawn_target(commands, materials, pos);
            }
        }
    }
}

fn load_next_level(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    wall_query: Query<Entity, With<Wall>>,
    target_query: Query<Entity, With<Target>>,
    mut level_info: ResMut<LevelInfo>,
    mut reader: EventReader<NextLevelEvent>,
) {
    if reader.iter().next().is_some() {
        // unload all walls
        for wall in wall_query.iter() {
            commands.entity(wall).despawn();
        }

        for target in target_query.iter() {
            commands.entity(target).despawn();
        }

        level_info.current_level += 1;
        level_info.counter_completion = 0;

        spawn_border(&mut commands, &mut materials, &asset_server);
        load_level(
            &mut commands,
            &mut materials,
            &asset_server,
            level_info.current_level,
        );
    }
}

fn setup_scoreboard(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let control_w = asset_server.load("controls_w.png");
    let control_a = asset_server.load("controls_a.png");
    let control_s = asset_server.load("controls_s.png");
    let control_d = asset_server.load("controls_d.png");
    let control_r = asset_server.load("controls_r.png");

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(control_w.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Position {
            x: ARENA_WIDTH as i32 / 4 * 3,
            y: ARENA_HEIGHT as i32 + 1,
        })
        .insert(Size::square(0.8));
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(control_a.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Position {
            x: ARENA_WIDTH as i32 / 4 * 3 - 1,
            y: ARENA_HEIGHT as i32,
        })
        .insert(Size::square(0.8));
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(control_s.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Position {
            x: ARENA_WIDTH as i32 / 4 * 3,
            y: ARENA_HEIGHT as i32,
        })
        .insert(Size::square(0.8));
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(control_d.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Position {
            x: ARENA_WIDTH as i32 / 4 * 3 + 1,
            y: ARENA_HEIGHT as i32,
        })
        .insert(Size::square(0.8));
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(control_r.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Position {
            x: ARENA_WIDTH as i32 / 4 * 3 + 2,
            y: ARENA_HEIGHT as i32 + 1,
        })
        .insert(Size::square(0.8));

    commands
        .spawn_bundle(Text2dBundle {
            text: Text {
                sections: vec![TextSection {
                    value: "turns left: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/press-start/prstart.ttf"),
                        font_size: 20.0,
                        color: Color::rgb(0.125, 0.164, 0.266),
                    },
                }],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Position {
            x: ARENA_WIDTH as i32 / 3 + 1,
            y: ARENA_HEIGHT as i32,
        });
}

fn _setup_statusbar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
){
    let texture_handle = asset_server.load("status_bar.png");
        let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(32.0, 8.0), 4, 1);
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite {
                index: 0,
                ..Default::default()
            },
            ..Default::default()
        }).insert(Position {
            x: ARENA_WIDTH as i32 / 2,
            y: ARENA_HEIGHT as i32 + 1 ,
        });
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut level_info: ResMut<LevelInfo>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    // Add border walls
    level_info.current_level = 0;
    level_info.counter_completion = 0;
    spawn_border(&mut commands, &mut materials, &asset_server);
    load_level(
        &mut commands,
        &mut materials,
        &asset_server,
        level_info.current_level,
    );
}

fn spawn_rocket(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rocket_path: ResMut<RocketPath>,
) {
    let texture_handle = asset_server.load("LunarLander/Ships/Spaceships_green_4.png");
    let start_position = Position { x: 1, y: 1 };
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(texture_handle.into()),
            sprite: Sprite::new(Vec2::new(SPRITE_WIDTH as f32, SPRITE_HEIGHT as f32)),
            ..Default::default()
        })
        .insert(Rocket {
            direction: Direction::StandStill,
            turns_left: MAX_TURNS,
        })
        .insert(start_position.clone())
        .insert(Size::square(0.8))
        .id();
    rocket_path.0 = vec![start_position.clone()];
    rocket_path.1 = vec![];
}

fn rocket_movement_input(keyboard_input: Res<Input<KeyCode>>, mut rockets: Query<&mut Rocket>) {
    let right = keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D);
    let left = keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A);
    let up = keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::W);
    let down = keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::S);

    if let Some(mut rocket) = rockets.iter_mut().next() {
        if rocket.turns_left <= 0 {
            return;
        }
        let old_dir = rocket.direction.clone();
        let dir: Direction = if left {
            Direction::Left
        } else if down {
            Direction::Down
        } else if up {
            Direction::Up
        } else if right {
            Direction::Right
        } else {
            rocket.direction
        };
        if dir != rocket.direction.opposite() {
            rocket.direction = dir;
        }

        if rocket.direction != old_dir {
            rocket.turns_left -= 1;
        }
    }
}
fn rocket_movement(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut rocket_query: Query<(&Rocket, &mut Position)>,
    collider_query: Query<&Transform, With<Wall>>,
    target_query: Query<&Transform, With<Target>>,
    windows: Res<Windows>,
    mut target_writer: EventWriter<TargetEvent>,
    mut rocket_path: ResMut<RocketPath>,
    asset_server: Res<AssetServer>,
    mut level_info: ResMut<LevelInfo>,
) {
    let window = windows.get_primary().unwrap();
    if let Some((rocket, mut rocket_pos)) = rocket_query.iter_mut().next() {
        let mut next_position = rocket_pos.clone();

        match &rocket.direction {
            Direction::Left => {
                if rocket_pos.x > 0 {
                    next_position.x -= 1;
                }
            }
            Direction::Right => {
                if rocket_pos.x < ARENA_WIDTH as i32 - 1 {
                    next_position.x += 1;
                }
            }
            Direction::Up => {
                if rocket_pos.y < ARENA_HEIGHT as i32 - 1 {
                    next_position.y += 1;
                }
            }
            Direction::Down => {
                if rocket_pos.y > 0 {
                    next_position.y -= 1;
                }
            }
            Direction::StandStill => {}
        };

        let mut no_collision = true;

        for wall_transform in collider_query.iter() {
            let next_transform_x = convert_x(next_position.x, window.width());
            let next_transform_y = convert_y(next_position.y, window.height());
            if wall_transform.translation.x == next_transform_x
                && wall_transform.translation.y == next_transform_y
            {
                no_collision = false;
            }
        }
        if no_collision && rocket.direction != Direction::StandStill {
            rocket_pos.x = next_position.x;
            rocket_pos.y = next_position.y;
            rocket_path.0.push(rocket_pos.clone());

            if rocket_path.0.len() >= 3 {
                let previous = rocket_path.0[rocket_path.0.len() - 3];
                let middle = rocket_path.0[rocket_path.0.len() - 2];
                let next = rocket_path.0[rocket_path.0.len() - 1];

                if previous.x != next.x && previous.y != next.y {
                    rocket_path.1.push(spawn_wall(
                        &mut commands,
                        &mut materials,
                        &asset_server,
                        middle,
                    ));
                }
            }
        }

        for target_transform in target_query.iter() {
            let next_transform_x = convert_x(next_position.x, window.width());
            let next_transform_y = convert_y(next_position.y, window.height());
            if target_transform.translation.x == next_transform_x
                && target_transform.translation.y == next_transform_y
            {
                level_info.counter_completion += 1;
                target_writer.send(TargetEvent {});
            }
        }
    }
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size, mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / (ARENA_HEIGHT + SCORE_BOARD_HEIGHT) as f32
                * window.height() as f32,
        );
    }
}

fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
    let tile_size = bound_window / bound_game;
    pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
}

fn convert_x(pos: i32, window_width: f32) -> f32 {
    convert(pos as f32, window_width, ARENA_WIDTH as f32)
}

fn convert_y(pos: i32, window_height: f32) -> f32 {
    convert(
        pos as f32,
        window_height,
        (ARENA_HEIGHT + SCORE_BOARD_HEIGHT) as f32,
    )
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert_x(pos.x, window.width()),
            convert_y(pos.y, window.height()),
            0.0,
        );
    }
}

fn rotation_translation(mut q: Query<(&mut Transform, &Rocket)>) {
    for (mut transform, rocket) in q.iter_mut() {
        transform.rotation = match rocket.direction {
            Direction::Right => Quat::from_rotation_z(0.0),
            Direction::Down => Quat::from_rotation_z(-std::f32::consts::PI * 0.5),
            Direction::Left => Quat::from_rotation_z(std::f32::consts::PI),
            Direction::Up => Quat::from_rotation_z(std::f32::consts::PI * 0.5),
            _ => Quat::from_rotation_z(0.0),
        };
    }
}

fn reached_target(
    mut reader: EventReader<TargetEvent>,
    mut rocket_query: Query<(&mut Rocket, &mut Position)>,
    mut segments: ResMut<RocketPath>,
    mut next_level_writer: EventWriter<NextLevelEvent>,
    level_info: Res<LevelInfo>,
    mut find_path_event: EventWriter<FindPathEvent>,
) {
    if reader.iter().next().is_some() {
        //TODO: update score
        if let Some((mut rocket, mut rocket_pos)) = rocket_query.iter_mut().next() {
            rocket.direction = Direction::StandStill;
            rocket.turns_left = MAX_TURNS;
            rocket_pos.x = 1;
            rocket_pos.y = 1;
            segments.0.clear();
            segments.0.push(rocket_pos.clone());
            segments.1.clear();

            if level_info.counter_completion > 2 {
                next_level_writer.send(NextLevelEvent {});
            } else {
                find_path_event.send(FindPathEvent {});
            }
        }
    }
}

fn scoreboard_system(mut rocket_query: Query<&Rocket>, mut query: Query<&mut Text>) {
    if let Some(rocket) = rocket_query.iter_mut().next() {
        let mut text = query.single_mut().unwrap();
        text.sections[0].value = format!("turns left: {}", rocket.turns_left);
    }
}

fn reset_input(keyboard_input: Res<Input<KeyCode>>, mut reset_writer: EventWriter<ResetEvent>) {
    if keyboard_input.pressed(KeyCode::R) {
        reset_writer.send(ResetEvent {});
    }
}

fn reset_last_one(
    mut commands: Commands,
    mut reader: EventReader<ResetEvent>,
    mut target_writer: EventWriter<TargetEvent>,
    mut rocket_path: ResMut<RocketPath>,
) {
    if reader.iter().next().is_some() {
        for i in 0..rocket_path.1.len() {
            let wall = rocket_path.1[i];
            commands.entity(wall).despawn();
        }
        rocket_path.1.clear();

        target_writer.send(TargetEvent {});
    }
}

use petgraph::algo::dijkstra;
use petgraph::graph::{NodeIndex, UnGraph};

fn path_finder(
    wall_query: Query<&Position, With<Wall>>,
    target_query: Query<&Position, With<Target>>,
    mut reader: EventReader<FindPathEvent>,
    mut game_over_writer: EventWriter<GameOverEvent>,
) {
    if reader.iter().next().is_some() {
        for target_position in target_query.iter() {
            let target_node: u32 =
                target_position.x as u32 * ARENA_WIDTH + target_position.y as u32;
            let mut array = [[true; ARENA_HEIGHT as usize]; ARENA_WIDTH as usize];

            for wall_position in wall_query.iter() {
                array[wall_position.x as usize][wall_position.y as usize] = false;
            }

            let mut edges: Vec<(u32, u32)> = vec![];
            for x in 1..ARENA_WIDTH - 1 {
                for y in 1..ARENA_HEIGHT - 1 {
                    if array[x as usize][y as usize] && array[x as usize + 1][y as usize] {
                        edges.push((x * ARENA_WIDTH + y, (x + 1) * ARENA_WIDTH + y));
                    }
                    if array[x as usize][y as usize] && array[x as usize][y as usize + 1] {
                        edges.push((x * ARENA_WIDTH + y, x * ARENA_WIDTH + y + 1));
                    }
                }
            }

            let g = UnGraph::<i32, ()>::from_edges(&edges);

            // Find the shortest path from source to tarte using `1` as the cost for every edge.
            let node_map = dijkstra(
                &g,
                (ARENA_WIDTH + 1).into(),
                Some(target_node.into()),
                |_| 1,
            );
            if node_map.contains_key(&NodeIndex::new(target_node as usize)) {
                return;
            }
        }

        game_over_writer.send(GameOverEvent {});
    }
}

fn main() {
    let mut app = App::build();
    app.insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(WindowDescriptor {
            title: "BUTTERFLY EFFECT".to_string(),
            width: WINDOW_WIDTH as f32,
            height: WINDOW_HEIGHT as f32,
            ..Default::default()
        })
        .insert_resource(RocketPath::default())
        .insert_resource(LevelInfo::default())
        .add_startup_system(setup.system())
        .add_startup_system(setup_scoreboard.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_rocket.system()))
        .add_system(scoreboard_system.system())
        .add_system(
            rocket_movement_input
                .system()
                .label(RocketMovement::Input)
                .before(RocketMovement::Movement),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.050))
                .with_system(rocket_movement.system().label(RocketMovement::Movement)),
        )
        .add_system(
            reached_target
                .system()
                .label(RocketMovement::Target)
                .after(RocketMovement::Movement),
        )
        .add_system(
            reset_input
                .system()
                .label(RocketMovement::Reset)
                .after(RocketMovement::Movement),
        )
        .add_system(reset_last_one.system().after(RocketMovement::Reset))
        .add_system(
            load_next_level
                .system()
                .label(RocketMovement::Loading)
                .after(RocketMovement::Target),
        )
        .add_system(
            path_finder
                .system()
                .label(RocketMovement::Path)
                .after(RocketMovement::Loading)
                .after(RocketMovement::Reset),
        )
        .add_system(load_game_over.system().after(RocketMovement::Path))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation.system())
                .with_system(rotation_translation.system())
                .with_system(size_scaling.system()),
        )
        .add_event::<TargetEvent>()
        .add_event::<ResetEvent>()
        .add_event::<NextLevelEvent>()
        .add_event::<FindPathEvent>()
        .add_event::<GameOverEvent>()
        .add_plugins(DefaultPlugins);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    app.run();
}

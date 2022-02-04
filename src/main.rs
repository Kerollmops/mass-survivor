use bevy::math::Vec3Swizzles;
use bevy::prelude::*;
use rand::Rng;

const MAP_SIZE: u32 = 41;
const GRID_WIDTH: f32 = 0.05;
const ENNEMIES_SPEED: f32 = 0.9;

struct EnnemyWaveSpawned(bool);

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(spawn_player)
        .add_system(camera_follow)
        .add_system(move_player)
        .add_system(spawn_ennemies)
        .add_system(move_ennemies)
        .run();
}

fn setup(mut commands: Commands) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);
    commands.insert_resource(EnnemyWaveSpawned(false));

    // Horizontal lines
    for i in 0..=MAP_SIZE {
        commands.spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                0.,
                i as f32 - MAP_SIZE as f32 / 2.,
                10.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(MAP_SIZE as f32, GRID_WIDTH)),
                ..Default::default()
            },
            ..Default::default()
        });
    }

    // Vertical lines
    for i in 0..=MAP_SIZE {
        commands.spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(
                i as f32 - MAP_SIZE as f32 / 2.,
                0.,
                10.,
            )),
            sprite: Sprite {
                color: Color::rgb(0.27, 0.27, 0.27),
                custom_size: Some(Vec2::new(GRID_WIDTH, MAP_SIZE as f32)),
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

fn spawn_player(mut commands: Commands) {
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform::from_translation(Vec3::new(-2., 0., 100.)),
            sprite: Sprite {
                color: Color::rgb(0., 0.47, 1.),
                custom_size: Some(Vec2::new(1., 1.)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Player);
}

fn move_player(keys: Res<Input<KeyCode>>, mut player_query: Query<&mut Transform, With<Player>>) {
    for mut transform in player_query.iter_mut() {
        let mut direction = Vec2::ZERO;

        if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
            direction.y += 1.;
        }
        if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
            direction.y -= 1.;
        }
        if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
            direction.x += 1.;
        }
        if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            direction.x -= 1.;
        }

        if direction == Vec2::ZERO {
            continue;
        }

        let move_speed = 0.13;
        let move_delta = (direction * move_speed).extend(0.);

        transform.translation += move_delta;
    }
}

fn gen_in_radius<R: Rng>(rng: &mut R, center: Vec3, radius: f32, deadzone: f32) -> Vec2 {
    use std::f32::consts::PI;
    let [x0, y0, _] = center.to_array();
    let t = 2.0 * PI * rng.gen_range(0.0..=1.0);
    let r = radius * rng.gen_range(0.0..=1.0f32).sqrt() + deadzone;
    let x = x0 + r * t.cos();
    let y = y0 + r * t.sin();
    Vec2::new(x, y)
}

fn spawn_ennemies(
    time: Res<Time>,
    mut ennemy_wave_spawned: ResMut<EnnemyWaveSpawned>,
    player_query: Query<&Transform, With<Player>>,
    mut commands: Commands,
) {
    match time.time_since_startup().as_secs() {
        2 => {
            if let EnnemyWaveSpawned(false) = *ennemy_wave_spawned {
                let mut rng = rand::thread_rng();
                for player_transform in player_query.iter() {
                    for i in 0..40 {
                        let pos = gen_in_radius(&mut rng, player_transform.translation, 10.0, 2.0)
                            .extend(90.0);

                        commands
                            .spawn_bundle(SpriteBundle {
                                transform: Transform::from_translation(pos),
                                sprite: Sprite {
                                    color: Color::rgb(0., 0.9, 0.3),
                                    custom_size: Some(Vec2::new(0.3, 0.3)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .insert(Ennemy);
                    }
                }
                *ennemy_wave_spawned = EnnemyWaveSpawned(true);
            }
        }
        _ => (),
    }
}

fn move_ennemies(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut ennemies_query: Query<&mut Transform, (With<Ennemy>, Without<Player>)>,
) {
    let player_transform = match player_query.iter().next() {
        Some(transform) => transform,
        None => return,
    };

    for mut ennemy_transform in ennemies_query.iter_mut() {
        let direction = player_transform.translation.xy().extend(ennemy_transform.translation.z)
            - ennemy_transform.translation;
        ennemy_transform.translation += direction * time.delta_seconds();
    }
}

fn camera_follow(
    player_query: Query<&Transform, With<Player>>,
    mut camera_query: Query<&mut Transform, (With<Camera>, Without<Player>)>,
) {
    for player_transform in player_query.iter() {
        let pos = player_transform.translation;

        for mut transform in camera_query.iter_mut() {
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Ennemy;

use std::time::Duration;

use benimator::*;
use bevy::prelude::*;
use bevy_asset_loader::AssetLoader;
use bevy_tweening::*;
use heron::prelude::*;
use ordered_float::OrderedFloat;
use rand::Rng;
use wasm_bindgen::prelude::*;

use self::assets::*;
use self::game_collision::*;

mod assets;
mod game_collision;

const MAP_SIZE: u32 = 41;
const GRID_WIDTH: f32 = 0.05;
const PLAYER_SPEED: f32 = 3.0;
const UNITS_Z_INDEX: f32 = 90.0;
const CONVERTING_WEAPON_DISTANCE: f32 = 3.5;
const PINK_CONVERTING_WEAPON: Color = Color::rgba(1., 0.584, 0.753, 1.);
const INVULNERABLE_DURATION: Duration = Duration::from_millis(2 * 1000 + 500); // 2.5s

// For wasm-pack to be happy...
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_main() {
    main();
}

fn main() {
    let mut app = App::new();
    AssetLoader::new(MyStates::AssetLoading)
        .continue_to_state(MyStates::Next)
        .with_collection::<GameAssets>()
        .build(&mut app);

    app.add_event::<GameCollisionEvent>()
        .add_state(MyStates::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
        .insert_resource(ConvertingWeaponTimer(Timer::new(Duration::from_secs(2), false)))
        .add_plugins(DefaultPlugins)
        .add_plugin(AnimationPlugin::default())
        .add_plugin(TweeningPlugin)
        .add_plugin(PhysicsPlugin::default())
        .insert_resource(Gravity::from(Vec3::ZERO))
        .add_startup_system(setup)
        .add_system_to_stage(CoreStage::PostUpdate, camera_follow)
        .add_system_set(
            SystemSet::on_enter(MyStates::Next)
                .with_system(spawn_player)
                .with_system(spawn_ennemies),
        )
        .add_system_set(
            SystemSet::on_update(MyStates::Next)
                .with_system(produce_game_collision_events)
                .with_system(tick_invulnerable)
                .with_system(display_player_invulnerability)
                .with_system(follow_nearest_player)
                .with_system(follow_nearest_enemy)
                .with_system(move_player)
                .with_system(mark_player_as_taking_damage)
                .with_system(applying_player_damage)
                .with_system(move_converting_weapon_from_velocity)
                .with_system(mark_enemies_under_converting_weapon)
                .with_system(convert_enemies_under_converting_weapon)
                .with_system(change_love_animation)
                .with_system(change_health_animation)
                .with_system(animate_and_disable_dead_entities),
        )
        .add_system_to_stage(CoreStage::PostUpdate, delete_converting_weapon_animation)
        .add_system_to_stage(CoreStage::PostUpdate, change_animation_from_velocity)
        .add_system_to_stage(CoreStage::PostUpdate, reorder_sprite_units)
        .run();
}

fn setup(mut commands: Commands, mut animations: ResMut<Assets<SpriteSheetAnimation>>) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 1. / 50.;
    commands.spawn_bundle(camera_bundle);

    // generate the pink selector animations
    commands.insert_resource(PinkSelectorAnimationsSet {
        full: animations.add(PinkSelector::Full.animation()),
        two_third: animations.add(PinkSelector::TwoThird.animation()),
        one_third: animations.add(PinkSelector::OneThird.animation()),
        empty: animations.add(PinkSelector::Empty.animation()),
    });

    // generate the red selector animations
    commands.insert_resource(RedSelectorAnimationsSet {
        full: animations.add(RedSelector::Full.animation()),
        two_third: animations.add(RedSelector::TwoThird.animation()),
        one_third: animations.add(RedSelector::OneThird.animation()),
        empty: animations.add(RedSelector::Empty.animation()),
    });

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

fn spawn_player(
    mut commands: Commands,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    game_assets: Res<GameAssets>,
    red_selector_animations_set: Res<RedSelectorAnimationsSet>,
) {
    let character = Castle::SimpleMonk;
    let animations_set = CharactersAnimationsSet {
        idle: animations.add(character.idle_animation()),
        walk: animations.add(character.walk_animation()),
        attack: animations.add(character.attack_animation()),
        death: animations.add(character.death_animation().once()),
    };

    commands
        .spawn()
        .insert(Transform::from_translation(Vec3::new(0., 0., UNITS_Z_INDEX)))
        .insert(GlobalTransform::default())
        .insert(Velocity::default())
        .insert(RigidBody::Dynamic)
        .insert(CollisionShape::Cuboid {
            half_extends: Vec3::new(0.30, 0.25, 0.),
            border_radius: None,
        })
        .insert(RotationConstraints::lock())
        .insert(
            CollisionLayers::none()
                .with_group(GameLayer::Player)
                .with_masks(&[GameLayer::Ally, GameLayer::Enemy]),
        )
        .insert(Health::Full)
        .insert(TakingDamage::default())
        .insert(Invulnerable(Timer::new(INVULNERABLE_DURATION, false)))
        .insert(Player)
        .with_children(|parent| {
            // spawn the sprite
            parent
                .spawn_bundle(SpriteSheetBundle {
                    transform: Transform::from_translation(Vec3::new(0., 0.25, 0.)),
                    sprite: TextureAtlasSprite {
                        custom_size: Some(Vec2::new(1., 1.)),
                        ..Default::default()
                    },
                    texture_atlas: game_assets.castle.clone(),
                    ..Default::default()
                })
                // animation settings
                .insert(animations_set.idle.clone())
                .insert(animations_set)
                .insert(Play)
                .insert(MainEntitySprite);

            // spawn the pink selector on the player
            parent
                .spawn_bundle(SpriteSheetBundle {
                    transform: Transform::from_translation(Vec3::new(0., 1.1, 0.)),
                    sprite: TextureAtlasSprite {
                        custom_size: Some(Vec2::new(0.7, 0.7)),
                        ..Default::default()
                    },
                    texture_atlas: game_assets.red_selector_01.clone(),
                    ..Default::default()
                })
                .insert(red_selector_animations_set.full.clone())
                .insert(Play)
                .insert(RedSelectorHealth);

            // spawn the converting weapon
            parent
                .spawn()
                .insert(Transform::from_translation(Vec3::new(
                    CONVERTING_WEAPON_DISTANCE,
                    0.,
                    UNITS_Z_INDEX,
                )))
                .insert(GlobalTransform::default())
                .insert(RigidBody::Sensor)
                .insert(CollisionShape::Sphere { radius: 1.8 })
                .insert(RotationConstraints::lock())
                .insert(
                    CollisionLayers::none()
                        .with_group(GameLayer::ConvertingWeapon)
                        .with_masks(&[GameLayer::Ally, GameLayer::Enemy]),
                )
                .insert(ConvertingWeapon);
        });
}

fn spawn_ennemies(
    mut commands: Commands,
    _player_query: Query<&GlobalTransform, With<Player>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    game_assets: Res<GameAssets>,
) {
    let mut rng = rand::thread_rng();

    for x in -15..=-5 {
        for y in -5..=5 {
            let (texture_atlas, animations_set) = if rng.gen() {
                let elemental = Elemental::from_rng(&mut rng);
                let animation = CharactersAnimationsSet {
                    idle: animations.add(elemental.idle_animation()),
                    walk: animations.add(elemental.walk_animation()),
                    attack: animations.add(elemental.attack_animation()),
                    death: animations.add(elemental.death_animation().once()),
                };
                (game_assets.magic_elementals.clone(), animation)
            } else if rng.gen() {
                let inferno = Inferno::from_rng(&mut rng);
                let animation = CharactersAnimationsSet {
                    idle: animations.add(inferno.idle_animation()),
                    walk: animations.add(inferno.walk_animation()),
                    attack: animations.add(inferno.attack_animation()),
                    death: animations.add(inferno.death_animation().once()),
                };
                (game_assets.infernos.clone(), animation)
            } else {
                let necromancer = Necromancer::from_rng(&mut rng);
                let animation = CharactersAnimationsSet {
                    idle: animations.add(necromancer.idle_animation()),
                    walk: animations.add(necromancer.walk_animation()),
                    attack: animations.add(necromancer.attack_animation()),
                    death: animations.add(necromancer.death_animation().once()),
                };
                (game_assets.necromancers.clone(), animation)
            };

            commands
                .spawn()
                .insert(Transform::from_translation(Vec3::new(x as f32, y as f32, UNITS_Z_INDEX)))
                .insert(GlobalTransform::default())
                .insert(Velocity::default())
                .insert(Acceleration::default())
                .insert(RigidBody::Dynamic)
                .insert(Damping::from_linear(1.))
                .insert(CollisionShape::Cuboid {
                    half_extends: Vec3::new(0.32, 0.25, 0.),
                    border_radius: None,
                })
                .insert(RotationConstraints::lock())
                .insert(CollisionLayers::none().with_group(GameLayer::Enemy).with_masks(&[
                    GameLayer::Player,
                    GameLayer::Ally,
                    GameLayer::Enemy,
                    GameLayer::ConvertingWeapon,
                ]))
                .insert(Enemy)
                .insert(FollowNearest { max_speed: rng.gen_range(0.2..=1.2) })
                .insert(UnderConvertingWeapon(false))
                .with_children(|parent| {
                    parent
                        .spawn_bundle(SpriteSheetBundle {
                            transform: Transform::from_translation(Vec3::new(0., 0.25, 0.)),
                            sprite: TextureAtlasSprite {
                                custom_size: Some(Vec2::new(1., 1.)),
                                ..Default::default()
                            },
                            texture_atlas,
                            ..Default::default()
                        })
                        // animation settings
                        .insert(animations_set.idle.clone())
                        .insert(animations_set)
                        .insert(Play);
                });
        }
    }
}

fn reorder_sprite_units(mut shape_query: Query<&mut Transform, With<CollisionShape>>) {
    for mut transform in shape_query.iter_mut() {
        transform.translation[2] = UNITS_Z_INDEX - transform.translation[1] * 0.1;
    }
}

fn change_animation_from_velocity(
    parent_query: Query<(&Velocity, &Health, &Children)>,
    mut child_query: Query<(
        &mut TextureAtlasSprite,
        &mut Handle<SpriteSheetAnimation>,
        &CharactersAnimationsSet,
    )>,
) {
    for (velocity, health, children) in parent_query.iter() {
        for &child in children.iter() {
            if let Ok((mut sprite, mut animation, animations_set)) = child_query.get_mut(child) {
                if let Health::Empty = health {
                    *animation = animations_set.death.clone();
                } else {
                    if velocity.linear[0] > 0. {
                        sprite.flip_x = false;
                    } else if velocity.linear[0] < 0. {
                        sprite.flip_x = true;
                    }

                    if velocity.linear.length() < 0.1 {
                        *animation = animations_set.idle.clone();
                    } else {
                        *animation = animations_set.walk.clone();
                    }
                }
            }
        }
    }
}

fn move_converting_weapon_from_velocity(
    parent_query: Query<(&Velocity, &Children)>,
    mut child_query: Query<&mut Transform, With<ConvertingWeapon>>,
) {
    for (velocity, children) in parent_query.iter() {
        for &child in children.iter() {
            if let Ok(mut transform) = child_query.get_mut(child) {
                if velocity.linear[0] > 0. {
                    transform.translation[0] = CONVERTING_WEAPON_DISTANCE;
                } else if velocity.linear[0] < 0. {
                    transform.translation[0] = -CONVERTING_WEAPON_DISTANCE;
                } else if velocity.linear[1] != 0. {
                    transform.translation[0] = 0.;
                }

                if velocity.linear[1] > 0. {
                    transform.translation[1] = CONVERTING_WEAPON_DISTANCE;
                } else if velocity.linear[1] < 0. {
                    transform.translation[1] = -CONVERTING_WEAPON_DISTANCE;
                } else if velocity.linear[0] != 0. {
                    transform.translation[1] = 0.;
                }
            }
        }
    }
}

/// Find the nearest player by using the squared distance (faster to compute).
fn follow_nearest_player(
    player_query: Query<&GlobalTransform, With<Player>>,
    mut enemy_query: Query<
        (&Transform, &mut Velocity, &mut Acceleration, &FollowNearest),
        With<Enemy>,
    >,
) {
    for (transform, mut velocity, mut acceleration, follow) in enemy_query.iter_mut() {
        let max_speed = follow.max_speed;
        match player_query
            .iter()
            .min_by_key(|pt| OrderedFloat(pt.translation.distance_squared(transform.translation)))
        {
            Some(pt) => {
                let direction = (pt.translation - transform.translation).normalize_or_zero();
                let limit = Vec3::splat(max_speed);
                velocity.linear = velocity.linear.clamp(-limit, limit);
                acceleration.linear = direction * 7.;
            }
            None => acceleration.linear = Vec3::ZERO,
        }
    }
}

/// Find the nearest enemy by using the squared distance (faster to compute).
fn follow_nearest_enemy(
    enemy_query: Query<&GlobalTransform, With<Enemy>>,
    mut ally_query: Query<
        (&Transform, &mut Velocity, &mut Acceleration, &FollowNearest),
        With<Ally>,
    >,
) {
    for (transform, mut velocity, mut acceleration, follow) in ally_query.iter_mut() {
        let max_speed = follow.max_speed;
        match enemy_query
            .iter()
            .min_by_key(|pt| OrderedFloat(pt.translation.distance_squared(transform.translation)))
        {
            Some(pt) => {
                let direction = (pt.translation - transform.translation).normalize_or_zero();
                let limit = Vec3::splat(max_speed);
                velocity.linear = velocity.linear.clamp(-limit, limit);
                acceleration.linear = direction * 7.;
            }
            None => acceleration.linear = Vec3::ZERO,
        }
    }
}

fn move_player(
    keys: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut Velocity, &Health), With<Player>>,
) {
    for (mut velocity, health) in player_query.iter_mut() {
        if *health != Health::Empty {
            let y = if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
                1.
            } else if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
                -1.
            } else {
                0.
            };

            let x = if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
                1.
            } else if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
                -1.
            } else {
                0.
            };

            velocity.linear = Vec2::new(x, y).normalize_or_zero().extend(0.) * PLAYER_SPEED;
        }
    }
}

fn mark_player_as_taking_damage(
    mut events: EventReader<GameCollisionEvent>,
    mut player_query: Query<&mut TakingDamage, With<Player>>,
) {
    let mut taking_damage = player_query.single_mut();

    for event in events.iter() {
        if let GameCollisionEvent::PlayerAndEnemy { status, .. } = event {
            match status {
                CollisionStatus::Started => taking_damage.count += 1,
                CollisionStatus::Stopped => {
                    taking_damage.count = taking_damage.count.saturating_sub(1)
                }
            }
        }
    }
}

fn applying_player_damage(
    mut player_query: Query<(&TakingDamage, &mut Invulnerable, &mut Health), With<Player>>,
) {
    let (taking_damage, mut invulnerable, mut health) = player_query.single_mut();

    if invulnerable.0.finished() {
        for _ in 0..taking_damage.count {
            invulnerable.0.reset();
            *health = health.take_damage();
        }
    }
}

fn animate_and_disable_dead_entities(
    mut query: Query<(&Health, &mut RigidBody, &Children)>,
    mut child_query: Query<(&mut Handle<SpriteSheetAnimation>, &CharactersAnimationsSet)>,
) {
    for (health, mut rigid_body, children) in query.iter_mut() {
        if let Health::Empty = health {
            *rigid_body = RigidBody::Sensor;
            for &child in children.iter() {
                if let Ok((mut animation, animations_set)) = child_query.get_mut(child) {
                    if *animation != animations_set.death {
                        *animation = animations_set.death.clone();
                    }
                }
            }
        }
    }
}

fn mark_enemies_under_converting_weapon(
    mut events: EventReader<GameCollisionEvent>,
    mut enemies_query: Query<&mut UnderConvertingWeapon, With<Enemy>>,
) {
    use GameCollisionEvent::ConvertingWeaponAndEnemy;

    for event in events.iter() {
        if let ConvertingWeaponAndEnemy { status, enemy, .. } = event {
            if let Ok(mut under_converting_weapon) = enemies_query.get_mut(*enemy) {
                match status {
                    CollisionStatus::Started => under_converting_weapon.0 = true,
                    CollisionStatus::Stopped => under_converting_weapon.0 = false,
                }
            }
        }
    }
}

fn convert_enemies_under_converting_weapon(
    time: Res<Time>,
    mut commands: Commands,
    mut converting_weapon_timer: ResMut<ConvertingWeaponTimer>,
    converting_weapon_query: Query<&GlobalTransform, With<ConvertingWeapon>>,
    mut enemies_query: Query<(Entity, &mut UnderConvertingWeapon), With<Enemy>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
    game_assets: Res<GameAssets>,
    pink_selector_animations_set: Res<PinkSelectorAnimationsSet>,
) {
    if converting_weapon_timer.0.tick(time.delta()).finished() {
        converting_weapon_timer.0.reset();

        // Spawn the one-time animation
        let global_transform = converting_weapon_query.single();
        let animation =
            SpriteSheetAnimation::from_range(0..=19, Duration::from_secs_f64(1.0 / 18.0)).once();
        commands
            .spawn_bundle(SpriteSheetBundle {
                transform: Transform::from_translation(global_transform.translation),
                sprite: TextureAtlasSprite {
                    color: PINK_CONVERTING_WEAPON,
                    custom_size: Some(Vec2::new(5., 5.)),
                    ..Default::default()
                },
                texture_atlas: game_assets.smoke_effect_07.clone(),
                ..Default::default()
            })
            .insert(animations.add(animation))
            .insert(Play)
            .insert(ConvertingWeaponAnimation);

        let mut rng = rand::thread_rng();
        for (entity, under_converting_weapon) in enemies_query.iter_mut() {
            if under_converting_weapon.0 && rng.gen::<f32>() < 0.2 {
                commands
                    .entity(entity)
                    .with_children(|parent| {
                        // Spawn the love animation
                        parent
                            .spawn_bundle(SpriteSheetBundle {
                                transform: Transform::from_translation(Vec3::new(0., 1.1, 0.)),
                                sprite: TextureAtlasSprite {
                                    custom_size: Some(Vec2::new(0.7, 0.7)),
                                    ..Default::default()
                                },
                                texture_atlas: game_assets.pink_selector_01.clone(),
                                ..Default::default()
                            })
                            .insert(pink_selector_animations_set.full.clone())
                            .insert(Play);
                    })
                    .insert(CollisionLayers::none().with_group(GameLayer::Ally).with_masks(&[
                        GameLayer::Player,
                        GameLayer::Ally,
                        GameLayer::Enemy,
                        GameLayer::ConvertingWeapon,
                    ]))
                    .insert(Ally)
                    .remove::<Enemy>();
            }
        }
    }
}

fn delete_converting_weapon_animation(
    mut commands: Commands,
    removals: RemovedComponents<Play>,
    animation_query: Query<&ConvertingWeaponAnimation>,
) {
    for entity in removals.iter() {
        if let Ok(_) = animation_query.get(entity) {
            commands.entity(entity).despawn();
        }
    }
}

fn change_love_animation(
    pink_selector_animations: Res<PinkSelectorAnimationsSet>,
    ally_query: Query<(&Children, &Health), (With<Ally>, Changed<Health>)>,
    mut child_query: Query<&mut Handle<SpriteSheetAnimation>>,
) {
    for (children, health) in ally_query.iter() {
        if let Ok(mut animation) = child_query.get_mut(children[0]) {
            *animation = match health {
                Health::Full => pink_selector_animations.full.clone(),
                Health::OneThird => pink_selector_animations.one_third.clone(),
                Health::TwoThird => pink_selector_animations.two_third.clone(),
                Health::Empty => pink_selector_animations.empty.clone(),
            };
        }
    }
}

fn change_health_animation(
    red_selector_animations: Res<RedSelectorAnimationsSet>,
    ally_query: Query<(&Children, &Health), (With<Player>, Changed<Health>)>,
    mut child_query: Query<&mut Handle<SpriteSheetAnimation>, With<RedSelectorHealth>>,
) {
    for (children, health) in ally_query.iter() {
        for child in children.iter() {
            if let Ok(mut animation) = child_query.get_mut(*child) {
                *animation = match health {
                    Health::Full => red_selector_animations.full.clone(),
                    Health::OneThird => red_selector_animations.one_third.clone(),
                    Health::TwoThird => red_selector_animations.two_third.clone(),
                    Health::Empty => red_selector_animations.empty.clone(),
                };
            }
        }
    }
}

fn tick_invulnerable(time: Res<Time>, mut player_query: Query<&mut Invulnerable, With<Player>>) {
    for mut invulnerable in player_query.iter_mut() {
        invulnerable.0.tick(time.delta());
    }
}

fn display_player_invulnerability(
    mut player_query: Query<(&Invulnerable, &Children), With<Player>>,
    mut child_query: Query<&mut TextureAtlasSprite, With<MainEntitySprite>>,
) {
    for (invulnerable, children) in player_query.iter_mut() {
        for child in children.iter() {
            if let Ok(mut texture_atlas_sprite) = child_query.get_mut(*child) {
                if invulnerable.0.finished() {
                    texture_atlas_sprite.color = Color::default();
                } else {
                    texture_atlas_sprite.color = Color::RED;
                }
            }
        }
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

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum MyStates {
    AssetLoading,
    Next,
}

#[derive(Component)]
pub struct CharactersAnimationsSet {
    idle: Handle<SpriteSheetAnimation>,
    walk: Handle<SpriteSheetAnimation>,
    attack: Handle<SpriteSheetAnimation>,
    death: Handle<SpriteSheetAnimation>,
}

#[derive(Component)]
pub struct PinkSelectorAnimationsSet {
    full: Handle<SpriteSheetAnimation>,
    two_third: Handle<SpriteSheetAnimation>,
    one_third: Handle<SpriteSheetAnimation>,
    empty: Handle<SpriteSheetAnimation>,
}

#[derive(Component)]
pub struct RedSelectorAnimationsSet {
    full: Handle<SpriteSheetAnimation>,
    two_third: Handle<SpriteSheetAnimation>,
    one_third: Handle<SpriteSheetAnimation>,
    empty: Handle<SpriteSheetAnimation>,
}

#[derive(Component)]
pub struct RedSelectorHealth;

#[derive(Component)]
pub struct MainEntitySprite;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct Ally;

#[derive(Copy, Clone, PartialEq, Eq, Component)]
pub enum Health {
    Full,
    TwoThird,
    OneThird,
    Empty,
}

impl Health {
    fn take_damage(&self) -> Health {
        match self {
            Health::Full => Health::TwoThird,
            Health::TwoThird => Health::OneThird,
            Health::OneThird => Health::Empty,
            Health::Empty => Health::Empty,
        }
    }
}

#[derive(Default, Component)]
pub struct TakingDamage {
    count: usize,
}

#[derive(Default, Component)]
pub struct Player;

#[derive(Component)]
pub struct Invulnerable(Timer);

#[derive(Component)]
pub struct Gem;

#[derive(Component)]
pub struct FollowNearest {
    max_speed: f32,
}

impl Default for FollowNearest {
    fn default() -> FollowNearest {
        FollowNearest { max_speed: 0.5 }
    }
}

#[derive(Component)]
pub struct ConvertingWeapon;

#[derive(Component)]
pub struct ConvertingWeaponTimer(Timer);

#[derive(Component)]
pub struct ConvertingWeaponAnimation;

#[derive(Component)]
pub struct UnderConvertingWeapon(bool);

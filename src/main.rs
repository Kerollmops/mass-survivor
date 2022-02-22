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
const CHARMING_PINK: Color = Color::rgba(1., 0.584, 0.753, 1.);
const INVULNERABLE_DURATION: Duration = Duration::from_millis(2 * 1000 + 500); // 2.5s
const CHARMED_DURATION: Duration = Duration::from_millis(25 * 1000); // 25s
const CHARMING_AREA_COOLDOWN: Duration = Duration::from_secs(2);
const LUCK_AT_CHARMING: f32 = 0.2;

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
        .add_event::<AllyEnemyConvertionEvent>()
        .add_state(MyStates::AssetLoading)
        .insert_resource(ClearColor(Color::rgb(0.53, 0.53, 0.53)))
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
                .with_system(display_invulnerability)
                .with_system(mark_charmed_eligibles)
                .with_system(tick_charmed)
                .with_system(tick_charming_areas)
                .with_system(charm_enemies)
                .with_system(refresh_allies_charming)
                .with_system(uncharm_allies)
                .with_system(hide_charming_animation)
                .with_system(mark_charmed_eligibles)
                .with_system(change_charmed_animation)
                .with_system(follow_nearest_player)
                .with_system(follow_nearest_enemy)
                .with_system(move_player)
                .with_system(mark_player_as_taking_damage)
                .with_system(applying_player_damage)
                .with_system(change_health_animation)
                .with_system(animate_and_disable_dead_entities),
        )
        .add_system_to_stage(CoreStage::PostUpdate, hide_charming_animation)
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
        .insert(Invulnerable {
            active_until: {
                let mut timer = Timer::new(INVULNERABLE_DURATION, false);
                timer.tick(INVULNERABLE_DURATION);
                timer
            },
        })
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

            // spawn the player health
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

            // spawn the converting weapon and the charming area but make it invisible
            parent
                .spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        color: CHARMING_PINK,
                        custom_size: Some(Vec2::new(5., 5.)),
                        ..Default::default()
                    },
                    texture_atlas: game_assets.smoke_effect_01.clone(),
                    visibility: Visibility { is_visible: false },
                    ..Default::default()
                })
                .insert(RigidBody::Sensor)
                .insert(CollisionShape::Sphere { radius: 1.8 })
                .insert(RotationConstraints::lock())
                .insert(
                    CollisionLayers::none()
                        .with_group(GameLayer::CharmingArea)
                        .with_masks(&[GameLayer::Ally, GameLayer::Enemy]),
                )
                .insert(CharmingArea { active_in: Timer::new(CHARMING_AREA_COOLDOWN, false) })
                .insert(
                    animations.add(
                        SpriteSheetAnimation::from_range(
                            0..=15,
                            Duration::from_secs_f64(1.0 / 18.0),
                        )
                        .once(),
                    ),
                );
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
                    GameLayer::CharmingArea,
                ]))
                .insert(Enemy)
                .insert(FollowNearest { max_speed: rng.gen_range(0.2..=1.2) })
                .insert(CharmedEligible { is_eligible: false })
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
    parent_query: Query<(&Velocity, Option<&Health>, &Children)>,
    mut child_query: Query<(
        &mut TextureAtlasSprite,
        &mut Handle<SpriteSheetAnimation>,
        &CharactersAnimationsSet,
    )>,
) {
    for (velocity, health, children) in parent_query.iter() {
        for &child in children.iter() {
            if let Ok((mut sprite, mut animation, animations_set)) = child_query.get_mut(child) {
                if health.map_or(false, |h| matches!(h, Health::Empty)) {
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

    if invulnerable.active_until.finished() {
        for _ in 0..taking_damage.count {
            invulnerable.active_until.reset();
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

fn tick_charming_areas(
    time: Res<Time>,
    mut commands: Commands,
    mut charming_areas: Query<(Entity, &mut Visibility, &mut CharmingArea)>,
    eligible_query: Query<(Entity, &CharmedEligible, Option<&Ally>, Option<&Enemy>)>,
    mut convertion_writer: EventWriter<AllyEnemyConvertionEvent>,
) {
    use AllyEnemyConvertionEvent::*;

    let (entity_area, mut area_visibility, mut charming_area) = charming_areas.single_mut();

    if charming_area.active_in.tick(time.delta()).finished() {
        area_visibility.is_visible = true;
        charming_area.active_in.reset();
        commands.entity(entity_area).insert(Play);

        let mut rng = rand::thread_rng();
        for (entity, eligible, ally, enemy) in eligible_query.iter() {
            if eligible.is_eligible && rng.gen::<f32>() < LUCK_AT_CHARMING {
                match (ally, enemy) {
                    (Some(_), _) => convertion_writer.send(AllyResetCharming(entity)),
                    (_, Some(_)) => convertion_writer.send(EnemyIntoAlly(entity)),
                    _ => (),
                }
            }
        }
    }
}

fn charm_enemies(
    mut commands: Commands,
    mut convertion_reader: EventReader<AllyEnemyConvertionEvent>,
    game_assets: Res<GameAssets>,
    pink_selector_animations_set: Res<PinkSelectorAnimationsSet>,
) {
    use AllyEnemyConvertionEvent::*;

    for convertion in convertion_reader.iter() {
        if let EnemyIntoAlly(entity) = convertion {
            commands
                .entity(*entity)
                .insert(CollisionLayers::none().with_group(GameLayer::Ally).with_masks(&[
                    GameLayer::Player,
                    GameLayer::Ally,
                    GameLayer::Enemy,
                    GameLayer::CharmingArea,
                ]))
                .insert(Charmed { active_until: Timer::new(CHARMED_DURATION, false) })
                .insert(Ally)
                .remove::<Enemy>()
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
                        .insert(CharmedAnimation)
                        .insert(Play);
                });
        }
    }
}

fn refresh_allies_charming(
    mut convertion_reader: EventReader<AllyEnemyConvertionEvent>,
    mut charmed_query: Query<&mut Charmed>,
) {
    use AllyEnemyConvertionEvent::*;

    for convertion in convertion_reader.iter() {
        if let AllyResetCharming(entity) = convertion {
            if let Ok(mut charmed) = charmed_query.get_mut(*entity) {
                charmed.active_until.reset();
            }
        }
    }
}

fn uncharm_allies(
    mut commands: Commands,
    mut convertion_reader: EventReader<AllyEnemyConvertionEvent>,
    mut charmed_query: Query<(Entity, &Children), With<Charmed>>,
    mut charmed_anim_query: Query<(), With<CharmedAnimation>>,
) {
    use AllyEnemyConvertionEvent::*;

    for convertion in convertion_reader.iter() {
        if let AllyIntoEnemy(entity) = convertion {
            if let Ok((entity, children)) = charmed_query.get_mut(*entity) {
                let mut entity_command = commands.entity(entity);

                entity_command
                    .insert(CollisionLayers::none().with_group(GameLayer::Enemy).with_masks(&[
                        GameLayer::Player,
                        GameLayer::Ally,
                        GameLayer::Enemy,
                        GameLayer::CharmingArea,
                    ]))
                    .remove::<Ally>()
                    .remove::<Charmed>()
                    .insert(Enemy);

                match children.iter().filter(|e| charmed_anim_query.get(**e).is_ok()).next() {
                    Some(entity) => {
                        entity_command.remove_children(&[*entity]);
                        commands.entity(*entity).despawn();
                    }
                    None => (),
                }
            }
        }
    }
}

fn hide_charming_animation(
    removals: RemovedComponents<Play>,
    mut animation_query: Query<&mut Visibility, With<CharmingArea>>,
) {
    for entity in removals.iter() {
        if let Ok(mut visibility) = animation_query.get_mut(entity) {
            visibility.is_visible = false;
        }
    }
}

fn mark_charmed_eligibles(
    mut events: EventReader<GameCollisionEvent>,
    mut query: Query<&mut CharmedEligible>,
) {
    use GameCollisionEvent::CharmedEnemy;

    for event in events.iter() {
        if let CharmedEnemy { status, enemy, .. } = event {
            if let Ok(mut charmed_eligible) = query.get_mut(*enemy) {
                match status {
                    CollisionStatus::Started => charmed_eligible.is_eligible = true,
                    CollisionStatus::Stopped => charmed_eligible.is_eligible = false,
                }
            }
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

fn tick_invulnerable(time: Res<Time>, mut invulnerable_query: Query<&mut Invulnerable>) {
    for mut invulnerable in invulnerable_query.iter_mut() {
        invulnerable.active_until.tick(time.delta());
    }
}

fn display_invulnerability(
    mut invulnerable_query: Query<(&Invulnerable, &Children)>,
    mut child_query: Query<&mut TextureAtlasSprite, With<MainEntitySprite>>,
) {
    for (invulnerable, children) in invulnerable_query.iter_mut() {
        for child in children.iter() {
            if let Ok(mut texture_atlas_sprite) = child_query.get_mut(*child) {
                if invulnerable.active_until.finished() {
                    texture_atlas_sprite.color = Color::default();
                } else {
                    texture_atlas_sprite.color = Color::RED;
                }
            }
        }
    }
}

fn tick_charmed(
    time: Res<Time>,
    mut charmed_query: Query<(Entity, &mut Charmed)>,
    mut convertion_writer: EventWriter<AllyEnemyConvertionEvent>,
) {
    use AllyEnemyConvertionEvent::*;

    for (entity, mut charmed) in charmed_query.iter_mut() {
        if charmed.active_until.tick(time.delta()).finished() {
            convertion_writer.send(AllyIntoEnemy(entity));
        }
    }
}

fn change_charmed_animation(
    charmed_query: Query<(&Charmed, &Children)>,
    mut child_query: Query<&mut Handle<SpriteSheetAnimation>, With<CharmedAnimation>>,
    pink_selector_animations: Res<PinkSelectorAnimationsSet>,
) {
    for (charmed, children) in charmed_query.iter() {
        let new_animation = if charmed.active_until.percent_left() > 0.75 {
            &pink_selector_animations.full
        } else if charmed.active_until.percent_left() > 0.50 {
            &pink_selector_animations.two_third
        } else if charmed.active_until.percent_left() > 0.25 {
            &pink_selector_animations.one_third
        } else {
            &pink_selector_animations.empty
        };

        for child in children.iter() {
            if let Ok(mut animation) = child_query.get_mut(*child) {
                *animation = new_animation.clone();
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
pub struct Invulnerable {
    active_until: Timer,
}

#[derive(Component)]
pub struct Charmed {
    active_until: Timer,
}

#[derive(Component)]
pub struct CharmedAnimation;

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
pub struct CharmingArea {
    active_in: Timer,
}

#[derive(Component)]
pub struct CharmedEligible {
    is_eligible: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum AllyEnemyConvertionEvent {
    AllyResetCharming(Entity),
    EnemyIntoAlly(Entity),
    AllyIntoEnemy(Entity),
}

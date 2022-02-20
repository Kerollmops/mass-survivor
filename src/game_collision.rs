use bevy::prelude::*;
use heron::prelude::*;

#[derive(Debug, Copy, Clone)]
pub enum GameCollisionEvent {
    PlayerAndEnemy { status: CollisionStatus, player: Entity, enemy: Entity },
    AllyAndEnemy { status: CollisionStatus, ally: Entity, enemy: Entity },
    EnemyAndEnemy(CollisionStatus, Entity, Entity),
    ConvertingWeaponAndEnemy { status: CollisionStatus, converting_weapon: Entity, enemy: Entity },
}

#[derive(Debug, Copy, Clone)]
pub enum CollisionStatus {
    Started,
    Stopped,
}

#[derive(PhysicsLayer)]
pub enum GameLayer {
    Player,
    ConvertingWeapon,
    Ally,
    Enemy,
}

pub fn produce_game_collision_events(
    mut in_events: EventReader<CollisionEvent>,
    mut out_events: EventWriter<GameCollisionEvent>,
) {
    use CollisionStatus::*;
    use GameCollisionEvent::*;

    for event in in_events.iter() {
        let (entity_1, entity_2) = event.rigid_body_entities();
        let (layers_1, layers_2) = event.collision_layers();
        let status = if event.is_started() { Started } else { Stopped };

        // player and enemy collide
        if is_player_layer(layers_1) && is_enemy_layer(layers_2) {
            out_events.send(PlayerAndEnemy { status, player: entity_1, enemy: entity_2 });
        } else if is_player_layer(layers_2) && is_enemy_layer(layers_1) {
            out_events.send(PlayerAndEnemy { status, player: entity_2, enemy: entity_1 });
        // ally and enemy collide
        } else if is_ally_layer(layers_1) && is_enemy_layer(layers_2) {
            out_events.send(AllyAndEnemy { status, ally: entity_1, enemy: entity_2 });
        } else if is_ally_layer(layers_2) && is_enemy_layer(layers_1) {
            out_events.send(AllyAndEnemy { status, ally: entity_2, enemy: entity_1 });
        // enemy and enemy collide
        } else if is_enemy_layer(layers_1) && is_enemy_layer(layers_2) {
            out_events.send(EnemyAndEnemy(status, entity_1, entity_2));
        // converting weapon and enemy collide
        } else if is_converting_weapon_layer(layers_1) && is_enemy_layer(layers_2) {
            out_events.send(ConvertingWeaponAndEnemy {
                status,
                converting_weapon: entity_1,
                enemy: entity_2,
            });
        } else if is_converting_weapon_layer(layers_2) && is_enemy_layer(layers_1) {
            out_events.send(ConvertingWeaponAndEnemy {
                status,
                converting_weapon: entity_2,
                enemy: entity_1,
            });
        }
    }
}

fn is_player_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Player)
}

fn is_ally_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Ally)
}

fn is_enemy_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::Enemy)
}

fn is_converting_weapon_layer(layers: CollisionLayers) -> bool {
    layers.contains_group(GameLayer::ConvertingWeapon)
}

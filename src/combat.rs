use bevy::{prelude::*, utils::hashbrown::HashMap};

use crate::{player::PlayerEntity, GameState};

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<CombatStats>()
            .register_type::<WantsToMelee>()
            .register_type::<SufferDamage>()
            .add_systems(
                Update,
                (
                    melee_combat.run_if(in_state(GameState::Playing)),
                    apply_damage.run_if(in_state(GameState::Playing)),
                    delete_the_dead.run_if(in_state(GameState::Playing)),
                ),
            );
    }
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct CombatStats {
    pub max_hp: i32,
    pub hp: i32,
    pub defense: i32,
    pub power: i32,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct WantsToMelee {
    pub target: Entity,
}

#[derive(Component, Debug, Clone, Reflect)]
pub struct SufferDamage {
    pub amount: Vec<i32>,
}

pub fn melee_combat(
    mut commands: Commands,
    q_wants_to_melee: Query<(Entity, &Parent, &WantsToMelee)>,
    mut q_combat_stats: Query<(&CombatStats, &Name, Option<&mut SufferDamage>)>,
) {
    let mut damage_map: HashMap<Entity, Vec<i32>> = HashMap::default();

    for (entity, parent, wants_to_melee) in q_wants_to_melee.iter() {
        let (active, active_name, _) = q_combat_stats.get(parent.get()).unwrap();
        if active.hp < 0 {
            continue;
        }

        let (unactive, unactive_name, _) = q_combat_stats.get(wants_to_melee.target).unwrap();
        if unactive.hp < 0 {
            continue;
        }

        let damage = 0.max(active.power - unactive.defense);
        if damage > 0 {
            if let Some(suffer_damages) = damage_map.get_mut(&wants_to_melee.target) {
                suffer_damages.push(damage);
            } else {
                damage_map.insert(wants_to_melee.target, vec![damage]);
            }
        } else {
            info!("{} is unable to hurt {}", active_name, unactive_name);
        }

        commands.entity(entity).despawn_recursive();
    }

    damage_map.into_iter().for_each(|(entity, damages)| {
        let (_, name, suffer_damage) = q_combat_stats.get_mut(entity).unwrap();

        if let Some(mut suffer_damage) = suffer_damage {
            suffer_damage.amount.extend_from_slice(&damages);
        } else {
            info!("{} is damaged for {} hp", name, damages.iter().sum::<i32>());
            commands
                .entity(entity)
                .insert(SufferDamage { amount: damages });
        }
    });
}

pub fn apply_damage(
    mut commands: Commands,
    mut q_suffer_damage: Query<(Entity, &mut CombatStats, &SufferDamage)>,
) {
    q_suffer_damage
        .iter_mut()
        .for_each(|(entity, mut stats, damage)| {
            stats.hp -= damage.amount.iter().sum::<i32>();
            commands.entity(entity).remove::<SufferDamage>();
            // if stats.hp <= 0 {
            //     commands.entity(entity).despawn_recursive();
            // }
        });
}

pub fn delete_the_dead(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    player_entity: Res<PlayerEntity>,
    q_combat_stats: Query<(Entity, &mut CombatStats)>,
) {
    q_combat_stats
        .iter()
        .filter(|(_, stats)| stats.hp <= 0)
        .for_each(|(entity, _)| {
            if entity == player_entity.0 {
                commands.remove_resource::<PlayerEntity>();
                next_state.set(GameState::Menu);
            }
            commands.entity(entity).despawn_recursive();
        });
}

use bevy::prelude::*;

use crate::actions::Actions;
use crate::combat::{CombatStats, WantsToMelee};
use crate::loading::TextureAssets;
use crate::map::{spawn_map, BlockTile, Map, Position, Tile, Viewshed};
use crate::GameState;

pub struct PlayerPlugin;

#[derive(Component)]
pub struct Player;

/// This plugin handles player related stuff like movement
/// Player logic is only active during the State `GameState::Playing`
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player.after(spawn_map))
            .add_systems(Update, player_input.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_player(mut commands: Commands, texture_assets: Res<TextureAssets>, map: Res<Map>) {
    let player_pos = if map.rooms.is_empty() {
        (map.cols / 2, map.rows / 2)
    } else {
        map.rooms[0].center()
    };

    commands.spawn((
        SpriteSheetBundle {
            transform: Transform {
                translation: Vec3::new(player_pos.0 as f32 * 16.0, player_pos.1 as f32 * 16.0, 1.0),
                scale: Vec3::splat(1.0),
                ..default()
            },
            texture: texture_assets.map_atlas.clone(),
            atlas: TextureAtlas {
                index: 25,
                layout: map.tileset_atlas_layout.as_ref().unwrap().clone(),
            },
            ..default()
        },
        Player,
        Name::new("Player"),
        CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        },
        Position {
            x: player_pos.0,
            y: player_pos.1,
        },
        Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        },
    ));
}

pub fn player_input(
    mut commands: Commands,
    time: Res<Time>,
    map: Res<Map>,
    actions: Res<Actions>,
    q_combat_stats: Query<&mut CombatStats>,
    mut q_player: Query<(Entity, &mut Transform, &mut Position, &mut Viewshed), With<Player>>,
) {
    if actions.player_movement.is_none() {
        return;
    }
    let movement = actions.player_movement.unwrap();
    if let Ok((player_entity, mut player_transform, mut pos, mut viewshed)) =
        q_player.get_single_mut()
    {
        let x = movement.0.saturating_add(pos.x as i32);
        let y = movement.1.saturating_add(pos.y as i32);
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= map.cols || y >= map.rows {
            return;
        }

        let idx = map.xy_to_index(x, y);
        if !map.blocked[idx] {
            pos.x = x;
            pos.y = y;

            player_transform.translation = Vec3::new(
                pos.x as f32 * map.tile_size as f32,
                pos.y as f32 * map.tile_size as f32,
                1.0,
            );

            viewshed.dirty = true;
        } else {
            map.tile_content[idx].iter().for_each(|potential_target| {
                match q_combat_stats.get(*potential_target) {
                    Ok(_target) => {
                        // Attack!
                        commands.entity(player_entity).with_children(|parent| {
                            parent.spawn(WantsToMelee {
                                target: *potential_target,
                            });
                        });
                        // Don't move player after attacking
                        return;
                    }
                    Err(e) => {
                        //
                        error!(
                            "Tile content index error, entity is :{:?}",
                            potential_target
                        );
                        error!("Error: {}", e);
                    }
                }
            });
            warn!("Blocked at ({}, {})!", pos.x, pos.y);
        }
    }
}

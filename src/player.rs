use bevy::prelude::*;

use crate::actions::Actions;
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
            .add_systems(Update, move_player.run_if(in_state(GameState::Playing)));
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
        BlockTile,
        Name::new("Player"),
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

pub fn move_player(
    time: Res<Time>,
    map: Res<Map>,
    actions: Res<Actions>,
    mut player_query: Query<(&mut Transform, &mut Position, &mut Viewshed), With<Player>>,
) {
    if actions.player_movement.is_none() {
        return;
    }
    let movement = (
        actions.player_movement.unwrap().0,
        actions.player_movement.unwrap().1,
    );
    if let Ok((mut player_transform, mut pos, mut viewshed)) = player_query.get_single_mut() {
        let x = movement.0.saturating_add(pos.x as i32);
        let y = movement.1.saturating_add(pos.y as i32);
        if x as usize >= map.cols || y as usize >= map.rows || x < 0 || y < 0 {
            return;
        }

        let idx = map.xy_to_index(x as usize, y as usize);
        if !map.blocked[idx] {
            pos.x = x as usize;
            pos.y = y as usize;

            player_transform.translation = Vec3::new(
                pos.x as f32 * map.tile_size as f32,
                pos.y as f32 * map.tile_size as f32,
                1.0,
            );

            viewshed.dirty = true;
        }
    }
}

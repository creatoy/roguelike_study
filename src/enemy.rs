use bevy::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::loading::TextureAssets;
use crate::map::{spawn_map, Map, Position, Tile};
use crate::GameState;

pub struct EnemyPlugin;

#[derive(Component)]
pub struct Enemy;

#[derive(Resource)]
pub struct EnemyTimer(Timer);

/// This plugin handles enemy related stuff like movement
/// Enemy logic is only active during the State `GameState::Playing`
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EnemyTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
            .add_systems(OnEnter(GameState::Playing), spawn_enemy.after(spawn_map))
            .add_systems(Update, move_enemy.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_enemy(mut commands: Commands, texture_assets: Res<TextureAssets>, map: Res<Map>) {
    let mut rng = ThreadRng::default();

    map.rooms.iter().skip(1).for_each(|room| {
        let enemy_pos = room.center();

        commands.spawn((
            SpriteSheetBundle {
                transform: Transform {
                    translation: Vec3::new(
                        enemy_pos.0 as f32 * 16.0,
                        enemy_pos.1 as f32 * 16.0,
                        1.0,
                    ),
                    scale: Vec3::splat(1.0),
                    ..default()
                },
                texture: texture_assets.map_atlas.clone(),
                atlas: TextureAtlas {
                    index: rng.gen_range(5..=9) * 48 + rng.gen_range(24..32),
                    layout: map.tileset_atlas_layout.as_ref().unwrap().clone(),
                },
                ..default()
            },
            Enemy,
            Position {
                x: enemy_pos.0,
                y: enemy_pos.1,
            },
        ));
    });
}

fn move_enemy(
    time: Res<Time>,
    map: Res<Map>,
    mut enemy_timer: ResMut<EnemyTimer>,
    mut enemy_query: Query<(&mut Transform, &mut Position), With<Enemy>>,
) {
    if !enemy_timer.0.tick(time.delta()).finished() {
        return;
    }

    let mut rng = ThreadRng::default();

    enemy_query
        .iter_mut()
        .for_each(|(mut enemy_transform, mut pos)| {
            let movement = (rng.gen_range(-1i32..=1i32), rng.gen_range(-1i32..=1i32));

            let x = movement.0.saturating_add(pos.x as i32);
            let y = movement.1.saturating_add(pos.y as i32);
            if x as usize >= map.cols || y as usize >= map.rows || x < 0 || y < 0 {
                return;
            }
            match map.get_tile(x as usize, y as usize) {
                Tile::Floor => {
                    pos.x = x as usize;
                    pos.y = y as usize;

                    enemy_transform.translation = Vec3::new(
                        pos.x as f32 * map.tile_size as f32,
                        pos.y as f32 * map.tile_size as f32,
                        1.0,
                    );
                }
                _ => {}
            }
        });
}

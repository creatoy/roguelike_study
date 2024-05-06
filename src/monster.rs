use bevy::prelude::*;
use bracket_pathfinding::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::loading::TextureAssets;
use crate::map::{spawn_map, BlockTile, Map, Position, Viewshed};
use crate::player::Player;
use crate::GameState;

pub struct MonsterPlugin;

#[derive(Component)]
pub struct Monster;

#[derive(Resource)]
pub struct MonsterTimer(Timer);

/// This plugin handles monster related stuff like movement
/// Monster logic is only active during the State `GameState::Playing`
impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MonsterTimer(Timer::from_seconds(0.5, TimerMode::Repeating)))
            .add_systems(OnEnter(GameState::Playing), spawn_monster.after(spawn_map))
            .add_systems(Update, monster_ai.run_if(in_state(GameState::Playing)));
    }
}

fn spawn_monster(mut commands: Commands, texture_assets: Res<TextureAssets>, map: Res<Map>) {
    let mut rng = ThreadRng::default();

    map.rooms.iter().enumerate().skip(1).for_each(|(i, room)| {
        let monster_pos = room.center();

        commands.spawn((
            SpriteSheetBundle {
                transform: Transform {
                    translation: Vec3::new(
                        monster_pos.0 as f32 * 16.0,
                        monster_pos.1 as f32 * 16.0,
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
            Name::new(format!("Monster #{}", i)),
            Monster,
            BlockTile,
            Position {
                x: monster_pos.0,
                y: monster_pos.1,
            },
            Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            },
        ));
    });
}

fn monster_ai(
    mut set: ParamSet<(
        Query<
            (
                &mut Transform,
                &mut Position,
                &mut Viewshed,
                &Name,
                &Visibility,
            ),
            With<Monster>,
        >,
        Query<&Position, With<Player>>,
    )>,
    time: Res<Time>,
    map: Res<Map>,
    mut monster_timer: ResMut<MonsterTimer>,
) {
    if !monster_timer.0.tick(time.delta()).finished() {
        return;
    }

    let player = set.p1();
    let player_pos = player.single().clone();

    let mut monsters = set.p0();
    monsters
        .iter_mut()
        .for_each(|(mut transform, mut pos, mut viewshed, _name, visible)| {
            if viewshed.visible_tiles.contains(&player_pos.into()) {
                let distance = DistanceAlg::Pythagoras.distance2d(
                    Point::new(pos.x, pos.y),
                    Point::new(player_pos.x, player_pos.y),
                );
                if distance < 1.5 {
                    return;
                }

                let path = a_star_search(
                    map.xy_to_index(pos.x, pos.y),
                    map.xy_to_index(player_pos.x, player_pos.y),
                    &*map,
                );

                if path.success && path.steps.len() > 1 {
                    let next = path.steps[1] as usize;
                    pos.x = next % map.cols;
                    pos.y = next / map.cols;

                    transform.translation = Vec3::new(
                        pos.x as f32 * map.tile_size as f32,
                        pos.y as f32 * map.tile_size as f32,
                        1.0,
                    );

                    viewshed.dirty = true;
                }
            } else {
                // if monsters can't see player, player can't see monsters. so update the viewshed
                if *visible == Visibility::Visible {
                    viewshed.dirty = true;
                }
            }
        });
}

use bevy::math::vec3;
use bevy::prelude::*;

use crate::loading::TextureAssets;
use crate::GameState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map::new(64, 48, 16.))
            .add_systems(OnEnter(GameState::Playing), spawn_map)
            .add_systems(Update, update_map.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Tile {
    None,
    Floor,
    Wall,
}

#[derive(Component)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn to_translation(&self) -> Vec3 {
        Vec3::new(self.x, self.y, 0.0)
    }
}

#[derive(Resource)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Tile>,
    pub tile_size: f32,
    pub tileset_atlas: Option<Handle<TextureAtlasLayout>>,
}

impl Map {
    pub fn new(width: usize, height: usize, tile_size: f32) -> Self {
        Map {
            width,
            height,
            tiles: vec![Tile::None; width * height],
            tile_size,
            tileset_atlas: None,
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[y * self.width + x] = tile;
    }
}

fn spawn_map(
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    texture_assets: Res<TextureAssets>,
) {
    let rows = map.height;
    let cols = map.width;
    for r in 0..rows {
        for c in 0..cols {
            map.set_tile(c, r, Tile::Floor);
        }
    }

    let texture_atlas_layout = TextureAtlasLayout::from_grid(
        Vec2::new(map.tile_size, map.tile_size),
        map.width,
        map.height,
        None,
        None,
    );
    let tileset_atlas_handle = texture_atlases.add(texture_atlas_layout);
    // map.tileset_atlas = Some(tileset_atlas_handle);

    // commands.spawn(SpriteBundle {
    //     texture: texture_assets.icon_bevy.clone(),
    //     transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
    //     ..default()
    // });

    warn!("spawn_map");

    let floor_tile_idx = 1;
    for r in 0..rows {
        for c in 0..cols {
            commands.spawn(SpriteSheetBundle {
                transform: Transform {
                    translation: vec3(c as f32 * map.tile_size, r as f32 * map.tile_size, 1.0),
                    scale: Vec3::splat(1.0),
                    ..default()
                },
                texture: texture_assets.map_atlas.clone(),
                atlas: TextureAtlas {
                    index: floor_tile_idx,
                    layout: tileset_atlas_handle.clone(),
                },
                ..default()
            });
        }
    }
}

fn update_map(mut commands: Commands, map: Res<Map>, texture_assets: Res<TextureAssets>) {
    // if let Some(tileset_atlas) = &map.tileset_atlas {
    //     commands.spawn(SpriteSheetBundle  {
    //         transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     })
    // }
    // commands.spawn(SpriteBundle {
    //     texture: texture_assets.map_atlas.clone(),
    //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     ..default()
    // });
    // .insert(Map);
}

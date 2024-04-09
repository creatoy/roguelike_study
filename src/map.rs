use bevy::prelude::*;

use crate::loading::TextureAssets;
use crate::GameState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map::new(80, 45, 16))
            .add_systems(OnEnter(GameState::Playing), spawn_map)
            .add_systems(Update, update_map.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Tile {
    Floor,
    Wall,
}

impl Tile {
    pub fn index_in_sprite_sheet(&self) -> usize {
        match self {
            Tile::Floor => 0,
            Tile::Wall => 16,
        }
    }
}

#[derive(Component)]
pub struct TilePosition {
    pub col: usize,
    pub row: usize,
}

#[derive(Resource)]
pub struct Map {
    pub cols: usize,
    pub rows: usize,
    pub tiles: Vec<Tile>,
    pub tile_size: usize,
    pub tileset_atlas_layout: Option<Handle<TextureAtlasLayout>>,
}

impl Map {
    pub fn new(cols: usize, rows: usize, tile_size: usize) -> Self {
        Map {
            cols,
            rows,
            tiles: vec![Tile::Floor; cols * rows],
            tile_size,
            tileset_atlas_layout: None,
        }
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[y * self.cols + x] = tile;
    }
}

fn spawn_map(
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    texture_assets: Res<TextureAssets>,
) {
    let cols = map.cols;
    let rows = map.rows;

    // TODO: implement random map generator
    for r in 1..rows - 1 {
        for c in 1..cols - 1 {
            map.set_tile(
                c,
                r,
                if c % 10 == 0 || r % 10 == 0 {
                    if c % 15 == 0 || r % 15 == 0 {
                        Tile::Floor
                    } else {
                        Tile::Wall
                    }
                } else {
                    Tile::Floor
                },
            );
        }
    }
    for c in 0..cols {
        map.set_tile(c, 0, Tile::Wall);
        map.set_tile(c, rows - 1, Tile::Wall);
    }
    for r in 0..rows {
        map.set_tile(0, r, Tile::Wall);
        map.set_tile(cols - 1, r, Tile::Wall);
    }

    let tileset_texture_atlas_layout = TextureAtlasLayout::from_grid(
        Vec2::new(map.tile_size as f32, map.tile_size as f32),
        cols,
        rows,
        None,
        None,
    );
    let tileset_layout_handle = texture_atlas_layouts.add(tileset_texture_atlas_layout);
    map.tileset_atlas_layout = Some(tileset_layout_handle.clone());

    for r in 0..rows {
        for c in 0..cols {
            commands.spawn((
                SpriteSheetBundle {
                    transform: Transform {
                        translation: Vec3::new(
                            (c * map.tile_size) as f32,
                            (r * map.tile_size) as f32,
                            0.,
                        ),
                        // scale: Vec3::splat(1.0),
                        ..default()
                    },
                    texture: texture_assets.map_atlas.clone(),
                    atlas: TextureAtlas {
                        index: map.tiles[r * map.cols + c].index_in_sprite_sheet(),
                        layout: tileset_layout_handle.clone(),
                    },
                    ..default()
                },
                TilePosition { col: c, row: r },
            ));
        }
    }
}

fn update_map(mut tile: Query<(&mut TextureAtlas, &TilePosition)>, map: Res<Map>) {
    for (mut tile_atlas, tile_position) in &mut tile {
        tile_atlas.index =
            map.tiles[tile_position.row * map.cols + tile_position.col].index_in_sprite_sheet();
    }
}

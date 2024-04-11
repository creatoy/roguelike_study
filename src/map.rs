use bevy::prelude::*;
use bracket_pathfinding::prelude::*;
use rand::rngs::ThreadRng;
use rand::{random, Rng};

use crate::loading::TextureAssets;
use crate::player::Player;
use crate::GameState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map::new(80, 45, 16))
            .add_systems(OnEnter(GameState::Playing), spawn_map)
            .add_systems(PreUpdate, update_view.run_if(in_state(GameState::Playing)))
            .add_systems(Update, update_map.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
pub struct MapTile {
    pub col: usize,
    pub row: usize,
    pub visible: bool,
    pub revealed: bool,
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

pub struct Rect {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl Rect {
    pub fn new(x: usize, y: usize, w: usize, h: usize) -> Self {
        Rect {
            x1: x,
            y1: y,
            x2: x + w,
            y2: y + h,
        }
    }

    pub fn intersect(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }
}

#[derive(Resource)]
pub struct Map {
    pub cols: usize,
    pub rows: usize,
    pub tile_size: usize,
    pub tileset_atlas_layout: Option<Handle<TextureAtlasLayout>>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,

    tiles: Vec<Tile>,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == Tile::Wall
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.cols as i32, self.rows as i32)
    }
}

impl Map {
    pub fn new(cols: usize, rows: usize, tile_size: usize) -> Self {
        Map {
            cols,
            rows,
            tile_size,
            tileset_atlas_layout: None,
            tiles: vec![Tile::Wall; cols * rows],
            rooms: vec![],
            revealed_tiles: vec![false; cols * rows],
            visible_tiles: vec![false; cols * rows],
        }
    }

    pub fn set_tile(&mut self, col: usize, row: usize, tile: Tile) {
        self.tiles[row * self.cols + col] = tile;
    }

    pub fn get_tile(&self, col: usize, row: usize) -> Tile {
        self.tiles[row * self.cols + col]
    }

    pub fn set_rect(&mut self, rect: &Rect, tile: Tile) {
        for r in rect.y1..=rect.y2 {
            for c in rect.x1..=rect.x2 {
                self.set_tile(c, r, tile);
            }
        }
    }

    pub fn set_horizontal_line(&mut self, x1: usize, x2: usize, y: usize, tile: Tile) {
        for x in x1.min(x2)..=x1.max(x2) {
            self.set_tile(x, y, tile);
        }
    }

    pub fn set_vertical_line(&mut self, x: usize, y1: usize, y2: usize, tile: Tile) {
        for y in y1.min(y2)..=y1.max(y2) {
            self.set_tile(x, y, tile);
        }
    }

    pub fn xy_to_index(&self, x: usize, y: usize) -> usize {
        y * self.cols + x
    }
}

pub(crate) fn spawn_map(
    mut commands: Commands,
    mut map: ResMut<Map>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
    texture_assets: Res<TextureAssets>,
) {
    let cols = map.cols;
    let rows = map.rows;

    new_map_rooms_and_corridors(&mut map);

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
                        index: map.get_tile(c, r).index_in_sprite_sheet(),
                        layout: tileset_layout_handle.clone(),
                    },
                    ..default()
                },
                MapTile {
                    col: c,
                    row: r,
                    visible: false,
                    revealed: false,
                },
            ));
        }
    }
}

fn update_view(
    mut player_view: Query<(&mut Player, &mut Viewshed)>,
    mut tile: Query<&mut MapTile>,
    mut map: ResMut<Map>,
) {
    if let Ok((mut player, mut viewshed)) = player_view.get_single_mut() {
        if viewshed.dirty {
            viewshed.dirty = false;
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(
                Point::new(player.x as i32, player.y as i32),
                viewshed.range,
                &*map,
            );
            viewshed
                .visible_tiles
                .retain(|p| p.x >= 0 && p.x < map.cols as i32 && p.y >= 0 && p.y < map.rows as i32);

            for t in map.visible_tiles.iter_mut() {
                *t = false;
            }

            for pos in viewshed.visible_tiles.iter() {
                let idx = map.xy_to_index(pos.x as usize, pos.y as usize);

                map.revealed_tiles[idx] = true;
                map.visible_tiles[idx] = true;
            }
        }
    }
}

pub(crate) fn update_map(
    mut q_tile: Query<(
        &mut Handle<Image>,
        &mut TextureAtlas,
        &MapTile,
        &mut Visibility,
    )>,
    map: Res<Map>,
    texture_assets: Res<TextureAssets>,
) {
    if !map.is_changed() {
        return;
    }
    for (mut spritesheet, mut tile_atlas, tile, mut tile_visible) in &mut q_tile {
        tile_atlas.index = map.get_tile(tile.col, tile.row).index_in_sprite_sheet();

        *tile_visible = if map.revealed_tiles[map.xy_to_index(tile.col, tile.row)] {
            *spritesheet = if map.visible_tiles[map.xy_to_index(tile.col, tile.row)] {
                texture_assets.map_atlas.clone()
            } else {
                texture_assets.map_atlas_darkened.clone()
            };

            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn new_map_rooms_and_corridors(map: &mut Map) {
    let cols = map.cols;
    let rows = map.rows;

    map.set_horizontal_line(0, cols - 1, 0, Tile::Wall);
    map.set_horizontal_line(0, cols - 1, rows - 1, Tile::Wall);
    map.set_vertical_line(0, 0, rows - 1, Tile::Wall);
    map.set_vertical_line(cols - 1, 0, rows - 1, Tile::Wall);

    let mut rooms: Vec<Rect> = Vec::new();
    const MAX_ROOMS: usize = 20;
    const MIN_SIZE: usize = 6;
    const MAX_SIZE: usize = 10;

    let mut rng = ThreadRng::default();

    for _ in 0..MAX_ROOMS {
        let w = rng.gen_range(MIN_SIZE..MAX_SIZE);
        let h = rng.gen_range(MIN_SIZE..MAX_SIZE);
        let x = rng.gen_range(1..(cols - w - 1));
        let y = rng.gen_range(1..(rows - h - 1));

        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in &rooms {
            if new_room.intersect(other_room) {
                ok = false;
            }
        }

        if ok {
            map.set_rect(&new_room, Tile::Floor);

            if !rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rng.gen_range(0..2) == 1 {
                    map.set_horizontal_line(prev_x, new_x, prev_y, Tile::Floor);
                    map.set_vertical_line(new_x, prev_y, new_y, Tile::Floor);
                } else {
                    map.set_vertical_line(prev_x, prev_y, new_y, Tile::Floor);
                    map.set_horizontal_line(prev_x, new_x, new_y, Tile::Floor);
                }
            }

            rooms.push(new_room);
        }
    }

    map.rooms = rooms;
}

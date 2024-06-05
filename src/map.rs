use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::ReflectInspectorOptions;
use bevy_inspector_egui::InspectorOptions;
use bracket_pathfinding::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::loading::TextureAssets;
use crate::monster::Monster;
use crate::player::{player_input, Player};
use crate::GameState;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map::new(80, 45, 16))
            .register_type::<MapEntity>()
            .register_type::<Map>()
            .register_type::<Rect>()
            .register_type::<Position>()
            .add_systems(OnEnter(GameState::Playing), spawn_map)
            .add_systems(OnExit(GameState::Playing), clear_map)
            .add_systems(
                PreUpdate,
                update_view
                    .run_if(in_state(GameState::Playing))
                    .after(player_input),
            )
            .add_systems(Update, update_map.run_if(in_state(GameState::Playing)))
            .add_systems(Update, map_index.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Reflect, Debug, PartialEq, Clone, Copy)]
pub enum Tile {
    Floor,
    Wall,
}

impl Tile {
    pub fn index_in_sprite_sheet(&self) -> usize {
        match self {
            Tile::Floor => 2,
            Tile::Wall => 17 * 48 + 10,
        }
    }
}

#[derive(Resource, Reflect, Deref)]
pub struct MapEntity(Entity);

#[derive(Component)]
pub struct MapTile {
    pub col: usize,
    pub row: usize,
    pub visible: bool,
    pub revealed: bool,
}

#[derive(Component)]
pub struct BlockTile;

#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl From<Point> for Position {
    fn from(p: Point) -> Self {
        Position {
            x: p.x as usize,
            y: p.y as usize,
        }
    }
}

impl From<Position> for Point {
    fn from(p: Position) -> Self {
        Point::new(p.x as i32, p.y as i32)
    }
}

impl From<&Position> for Point {
    fn from(p: &Position) -> Self {
        Point::new(p.x as i32, p.y as i32)
    }
}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}

#[derive(Reflect, Clone, Copy)]
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

#[derive(Reflect, Resource, Default, Clone, InspectorOptions)]
#[reflect(Resource, InspectorOptions)]
// #[derive(Resource)]
pub struct Map {
    pub cols: usize,
    pub rows: usize,
    pub tile_size: usize,
    pub tileset_grids: Option<(usize, usize)>,
    pub rooms: Vec<Rect>,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub tile_content: Vec<Vec<Entity>>,

    tiles: Vec<Tile>,
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == Tile::Wall
    }

    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let mut exits = SmallVec::new();
        let x = idx % self.cols;
        let y = idx / self.cols;
        let c = self.cols;

        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0));
        }
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0));
        }
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - c, 1.0));
        }
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + c, 1.0));
        }

        if self.is_exit_valid(x - 1, y - 1) {
            exits.push((idx - c - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push((idx - c + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push((idx + c - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push((idx + c + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let c = self.cols;
        let p1 = Point::new(idx1 % c, idx1 / c);
        let p2 = Point::new(idx2 % c, idx2 / c);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
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
            tileset_grids: None,
            tiles: vec![Tile::Wall; cols * rows],
            rooms: vec![],
            revealed_tiles: vec![false; cols * rows],
            visible_tiles: vec![false; cols * rows],
            blocked: vec![false; cols * rows],
            tile_content: vec![Vec::new(); cols * rows],
        }
    }

    pub fn clear_map(&mut self) {
        self.tileset_grids = None;
        self.tiles.fill(Tile::Wall);
        self.rooms.clear();
        self.revealed_tiles.fill(false);
        self.visible_tiles.fill(false);
        self.blocked.fill(false);
        self.tile_content.iter_mut().for_each(|v| v.clear());
    }

    pub fn set_tile(&mut self, col: usize, row: usize, tile: Tile) {
        self.tiles[row * self.cols + col] = tile;
    }

    pub fn get_tile(&self, col: usize, row: usize) -> Tile {
        self.tiles[row * self.cols + col]
    }

    pub fn get_tile_index_in_sprite_sheet(&self, col: usize, row: usize) -> usize {
        if let Some((cols, _rows)) = self.tileset_grids {
            row * cols + col
        } else {
            0
        }
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

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == Tile::Wall;
        }
    }

    fn is_exit_valid(&self, x: usize, y: usize) -> bool {
        if x < 1 || x > self.cols - 1 || y < 1 || y > self.rows - 1 {
            return false;
        }

        let idx = self.xy_to_index(x, y);
        !self.blocked[idx]
    }

    fn clear_content_index(&mut self) {
        self.tile_content.iter_mut().for_each(|content| {
            content.clear();
        });
    }
}

pub(crate) fn spawn_map(
    mut commands: Commands,
    mut map: ResMut<Map>,
    texture_assets: Res<TextureAssets>,
    images: Res<Assets<Image>>,
) {
    let cols = map.cols;
    let rows = map.rows;

    new_map_rooms_and_corridors(&mut map);

    let map_atlas_image = images.get(&texture_assets.map_atlas).unwrap();
    let (atlas_cols, atlas_rows) = (
        map_atlas_image.width() as usize / map.tile_size,
        map_atlas_image.height() as usize / map.tile_size,
    );

    map.tileset_grids = Some((atlas_cols, atlas_rows));
    info!("Tileset grids: {:?}", map.tileset_grids);

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
                        layout: texture_assets.map_atlas_layout.clone(),
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

pub(crate) fn clear_map(mut commands: Commands, mut map: ResMut<Map>) {
    map.clear_map();
    // commands.run_system(update_map);
}

fn update_view(
    mut player_view: Query<(&mut Position, &mut Viewshed, &Name), With<Player>>,
    mut monster_view: Query<
        (&Position, &mut Viewshed, &mut Visibility, &Name),
        (With<Monster>, Without<Player>),
    >,
    mut map: ResMut<Map>,
) {
    if let Ok((pos, mut viewshed, _player_name)) = player_view.get_single_mut() {
        if viewshed.dirty {
            viewshed.dirty = false;
            viewshed.visible_tiles.clear();
            viewshed.visible_tiles = field_of_view(
                Point::new(pos.x as i32, pos.y as i32),
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

        monster_view
            .iter_mut()
            .for_each(|(e_pos, mut e_viewshed, mut e_visible, e_name)| {
                if e_viewshed.dirty {
                    e_viewshed.dirty = false;
                    e_viewshed.visible_tiles.clear();
                    e_viewshed.visible_tiles =
                        field_of_view(Point::new(e_pos.x, e_pos.y), e_viewshed.range, &*map);
                    e_viewshed.visible_tiles.retain(|p| {
                        p.x >= 0 && p.x < map.cols as i32 && p.y >= 0 && p.y < map.rows as i32
                    });

                    if map.visible_tiles[map.xy_to_index(e_pos.x as usize, e_pos.y as usize)] {
                        *e_visible = Visibility::Visible;
                        // info!("{} see {}", e_name, player_name);
                    } else {
                        *e_visible = Visibility::Hidden;
                    }
                }
            });
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
    q_tile.iter_mut().for_each(
        |(mut spritesheet, mut tile_atlas, tile, mut tile_visible)| {
            tile_atlas.index = map.get_tile(tile.col, tile.row).index_in_sprite_sheet();

            if map.revealed_tiles[map.xy_to_index(tile.col, tile.row)] {
                if map.visible_tiles[map.xy_to_index(tile.col, tile.row)] {
                    if *spritesheet != texture_assets.map_atlas {
                        *spritesheet = texture_assets.map_atlas.clone();
                    }
                } else {
                    if *spritesheet != texture_assets.map_atlas_darkened {
                        *spritesheet = texture_assets.map_atlas_darkened.clone();
                    }
                };

                if *tile_visible == Visibility::Hidden {
                    *tile_visible = Visibility::Visible;
                }
            } else {
                *tile_visible = Visibility::Hidden;
            }
        },
    );
}

pub fn map_index(
    mut map: ResMut<Map>,
    q_blocks: Query<&Position, With<BlockTile>>,
    q_position: Query<(Entity, &Position), With<Monster>>,
) {
    map.populate_blocked();
    map.clear_content_index();

    q_position.iter().for_each(|(entity, pos)| {
        let idx = map.xy_to_index(pos.x, pos.y);

        if let Ok(_) = q_blocks.get(entity) {
            map.blocked[idx] = true;
            map.tile_content[idx].push(entity);
        }
    });
}

fn update_blocks(q_blocks: Query<&Position, With<BlockTile>>, mut map: ResMut<Map>) {
    map.populate_blocked();

    q_blocks.iter().for_each(|pos| {
        let idx = map.xy_to_index(pos.x, pos.y);

        map.blocked[idx] = true;
    });
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

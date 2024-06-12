use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;

use crate::combat::CombatStats;
use crate::loading::TextureAssets;
use crate::map::{spawn_map, Map};
use crate::player::Player;
use crate::{GameState, HUD_ROWS};

pub struct GuiPlugin;
impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UiMaterialPlugin::<CustomUiMaterial>::default())
            .add_systems(OnEnter(GameState::Playing), setup_gui.after(spawn_map))
            .add_systems(
                Update,
                update_player_hp.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct GuiRoot;

#[derive(Component, Default, Clone, Copy)]
pub struct HudRoot;

#[derive(Component, Default, Clone, Copy)]
pub struct PlayerHpWidget;

fn setup_gui(
    mut commands: Commands,
    map: Res<Map>,
    texture_assets: Res<TextureAssets>,
    mut ui_materials: ResMut<Assets<CustomUiMaterial>>,
) {
    commands
        .spawn((
            NodeBundle {
                //transform: Transform::from_xyz(0.0, ((map.rows - 4) * map.tile_size) as f32, 10.0),
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::ColumnReverse,

                    ..default()
                },
                ..default()
            },
            GuiRoot,
        ))
        .with_children(|child| {
            child
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(HUD_ROWS * map.tile_size as f32),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Row,
                            ..default()
                        },
                        background_color: Color::BLACK.into(),
                        border_color: Color::WHITE.into(),
                        ..default()
                    },
                    Name::new("HudRoot"),
                    HudRoot,
                ))
                .with_children(|child| {
                    child.spawn((
                        TextBundle::from_section(
                            "Hp:",
                            TextStyle {
                                font_size: 16.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                                ..default()
                            },
                        ),
                        Name::new("Hp label"),
                    ));
                    child.spawn((
                        MaterialNodeBundle {
                            style: Style {
                                width: Val::Px(16.0),
                                height: Val::Px(16.0),
                                margin: UiRect::all(Val::Px(2.0)),
                                ..default()
                            },
                            material: ui_materials.add(CustomUiMaterial {
                                texture: texture_assets.map_atlas.clone(),
                                tiled: AtlasTiled::new(
                                    map.tileset_grids,
                                    (39, 10),
                                    Vec2::new(1.0, 1.0),
                                ),
                            }),
                            ..default()
                        },
                        // AtlasImageBundle {
                        //     style: Style {
                        //         width: Val::Px(16.0),
                        //         height: Val::Px(16.0),
                        //         margin: UiRect::all(Val::Px(2.0)),
                        //         ..default()
                        //     },
                        //     image: UiImage::new(texture_assets.map_atlas.clone()),
                        //     texture_atlas: TextureAtlas {
                        //         layout: texture_assets.map_atlas_layout.clone(),
                        //         index: 10 * map.tileset_grids.0 + 39,
                        //     },
                        //     ..default()
                        // },
                        // ImageBundle {
                        //     style: Style {
                        //         width: Val::Px(50.0 * 16.0),
                        //         height: Val::Px(16.0),
                        //         margin: UiRect::all(Val::Px(2.0)),
                        //         ..default()
                        //     },
                        //     image: UiImage::new(texture_assets.heart.clone()),
                        //     // image: UiImage::new(texture_assets.map_atlas.clone()),
                        //     ..default()
                        // },
                        // ImageScaleMode::Tiled {
                        //     tile_x: true,
                        //     tile_y: false,
                        //     stretch_value: 1.0,
                        // },
                        PlayerHpWidget,
                        Name::new("Heart icon"),
                    ));
                });
        });
}

fn update_player_hp(
    player_stats_q: Query<&CombatStats, (With<Player>, Changed<CombatStats>)>,
    mut ui_materials: ResMut<Assets<CustomUiMaterial>>,
    mut player_hp_widget_q: Query<(&mut Style, &Handle<CustomUiMaterial>), With<PlayerHpWidget>>,
) {
    if let Ok(stats) = player_stats_q.get_single() {
        if let Ok((mut style, material_handle)) = player_hp_widget_q.get_single_mut() {
            let material = ui_materials.get_mut(material_handle).unwrap();
            material.tiled.repeat.x = stats.hp as f32;
            style.width = Val::Px(stats.hp as f32 * 16.0);
        }
    }
}

#[derive(ShaderType, Debug, Clone)]
struct AtlasTiled {
    offset: Vec2,
    size: Vec2,
    repeat: Vec2,
}
impl AtlasTiled {
    pub fn new(atlas_grids: (usize, usize), pos: (usize, usize), repeat: Vec2) -> Self {
        Self {
            offset: Vec2::new(
                pos.0 as f32 / atlas_grids.0 as f32,
                pos.1 as f32 / atlas_grids.1 as f32,
            ),
            size: Vec2::new(1.0 / atlas_grids.0 as f32, 1.0 / atlas_grids.1 as f32),
            repeat,
        }
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
struct CustomUiMaterial {
    #[texture(0)]
    #[sampler(1)]
    texture: Handle<Image>,
    #[uniform(2)]
    tiled: AtlasTiled,
}

impl UiMaterial for CustomUiMaterial {
    // fn vertex_shader() -> ShaderRef {
    //     "shaders/ui_texture_atlas_tiled.wgsl".into()
    // }
    fn fragment_shader() -> ShaderRef {
        "shaders/ui_texture_atlas_tiled.wgsl".into()
    }
}

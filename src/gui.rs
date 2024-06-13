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
                                    Vec2::new(
                                        map.tileset_grids.0 as f32,
                                        map.tileset_grids.1 as f32,
                                    ),
                                    Vec2::new(16.0, 16.0),
                                    Vec2::new(39.0, 10.0),
                                ),
                            }),
                            ..default()
                        },
                        PlayerHpWidget,
                        Name::new("Heart icon"),
                    ));
                });
        });
}

fn update_player_hp(
    player_stats_q: Query<&CombatStats, (With<Player>, Changed<CombatStats>)>,
    mut player_hp_widget_q: Query<&mut Style, With<PlayerHpWidget>>,
) {
    if let Ok(stats) = player_stats_q.get_single() {
        if let Ok(mut style) = player_hp_widget_q.get_single_mut() {
            style.width = Val::Px(stats.hp as f32 * 16.0);
        }
    }
}

#[derive(ShaderType, Debug, Clone)]
struct AtlasTiled {
    atlas_grids: Vec2,
    tile_size: Vec2,
    tile_pos: Vec2,
}
impl AtlasTiled {
    pub fn new(atlas_grids: Vec2, tile_size: Vec2, tile_pos: Vec2) -> Self {
        Self {
            atlas_grids,
            tile_size,
            tile_pos,
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
    fn fragment_shader() -> ShaderRef {
        "shaders/ui_texture_atlas_tiled.wgsl".into()
    }
}

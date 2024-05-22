use bevy::prelude::*;

use crate::map::{spawn_map, Map};
use crate::GameState;

pub struct GuiPlugin;
impl Plugin for GuiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), setup_gui.after(spawn_map));
    }
}

#[derive(Component, Default, Clone, Copy)]
pub struct GuiRoot;

fn setup_gui(mut commands: Commands, map: Res<Map>) {
    commands.spawn((
        NodeBundle {
            // transform: Transform::from_xyz(0.0, ((map.rows - 4) * map.tile_size) as f32, 10.0),
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(4.0 * map.tile_size as f32),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            border_color: Color::WHITE.into(),
            ..default()
        },
        GuiRoot,
    ));
}

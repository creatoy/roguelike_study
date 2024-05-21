#![allow(clippy::type_complexity)]

mod actions;
mod audio;
mod combat;
mod loading;
mod map;
mod menu;
mod monster;
mod player;

use std::time::Duration;

use crate::actions::ActionsPlugin;
use crate::audio::InternalAudioPlugin;
use crate::loading::LoadingPlugin;
use crate::menu::MenuPlugin;
use crate::player::PlayerPlugin;

use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use combat::CombatPlugin;
use map::{Map, MapPlugin};
use monster::MonsterPlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum ProcessSet {}

#[derive(Component)]
struct FpsDiagnostic;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_plugins((
                LoadingPlugin,
                // MenuPlugin,
                ActionsPlugin,
                // InternalAudioPlugin,
                PlayerPlugin,
                MonsterPlugin,
                MapPlugin,
                CombatPlugin,
            ))
            .add_systems(Startup, setup_camera)
            .add_systems(
                Update,
                update_fps.run_if(on_timer(Duration::from_millis(200))),
            );

        #[cfg(debug_assertions)]
        {
            // app.add_plugins((FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin::default()));
        }
    }
}

fn setup_camera(mut commands: Commands, map: Res<Map>) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.transform.translation.x = (map.cols as f32 / 2.0 - 0.5) * map.tile_size as f32;
    camera_bundle.transform.translation.y = (map.rows as f32 / 2.0 - 0.5) * map.tile_size as f32;
    commands.spawn(camera_bundle);

    commands.spawn((
        Text2dBundle {
            transform: Transform::from_xyz(10.0, 10.0, 10.0),
            text: Text::from_section(
                "FPS: 0.0",
                TextStyle {
                    font_size: 24.0,
                    ..default()
                },
            ),
            text_anchor: bevy::sprite::Anchor::BottomLeft,
            ..default()
        },
        FpsDiagnostic,
    ));
}

fn update_fps(time: Res<Time<Real>>, mut query: Query<&mut Text, With<FpsDiagnostic>>) {
    let Ok(mut text) = query.get_single_mut() else {
        return;
    };

    text.sections[0].value = format!("FPS: {:.1}", 1.0 / time.delta_seconds());
    // text.sections
}

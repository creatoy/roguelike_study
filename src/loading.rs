use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_kira_audio::AudioSource;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Playing)
                .load_collection::<TextureAssets>(),
        );
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see <https://github.com/NiklasEi/bevy_asset_loader>)

#[derive(AssetCollection, Resource)]
pub struct AudioAssets {
    #[asset(path = "audio/flying.ogg")]
    pub flying: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
    #[asset(path = "textures/bevy.png")]
    pub icon_bevy: Handle<Image>,
    #[asset(path = "textures/github.png")]
    pub icon_github: Handle<Image>,
    #[asset(texture_atlas_layout(tile_size_x = 16., tile_size_y = 16., columns = 48, rows = 22))]
    pub map_atlas_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "textures/colored_packed.png")]
    pub map_atlas: Handle<Image>,
    #[asset(path = "textures/colored_packed_darkened.png")]
    pub map_atlas_darkened: Handle<Image>,
    #[asset(path = "textures/heart_red.png")]
    pub heart: Handle<Image>,
}

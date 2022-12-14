use crate::GameState;
use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

pub struct LoadingPlugin;

/// This plugin loads all assets using [AssetLoader] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at https://bevy-cheatbook.github.io/features/assets.html
impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading)
                .with_collection::<FontAssets>()
                .with_collection::<SpriteAssets>()
                .continue_to_state(GameState::MainMenu),
        );
    }
}

// the following asset collections will be loaded during the State `GameState::Loading`
// when done loading, they will be inserted as resources (see https://github.com/NiklasEi/bevy_asset_loader)
#[derive(AssetCollection)]
pub struct FontAssets {
    #[asset(path = "fonts/FiraSans-Bold.ttf")]
    pub fira_sans: Handle<Font>,
}

#[derive(AssetCollection)]
pub struct SpriteAssets {
    #[asset(path = "sprites/bomb.png")]
    pub bomb: Handle<Image>,

    #[asset(path = "sprites/castle.png")]
    pub castle: Handle<Image>,

    #[asset(path = "sprites/duck.png")]
    pub duck: Handle<Image>,

    #[asset(path = "sprites/magic.png")]
    pub magic: Handle<Image>,

    #[asset(path = "sprites/pistol.png")]
    pub pistol: Handle<Image>,

    #[asset(path = "sprites/shotgun.png")]
    pub shotgun: Handle<Image>,

    #[asset(path = "sprites/title.png")]
    pub title: Handle<Image>,
}

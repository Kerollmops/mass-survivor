use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
pub struct IconsetAssets {
    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 18, rows = 50))]
    #[asset(path = "images/iconset_fantasy_standalone.png")]
    pub iconset_fantasy_standalone: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 18, rows = 50))]
    #[asset(path = "images/iconset_fantasy_castshadows.png")]
    pub iconset_fantasy_castshadows: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 32., tile_size_y = 32., columns = 10, rows = 4))]
    #[asset(path = "images/iconset_halloween_standalone.png")]
    pub iconset_halloween_standalone: Handle<TextureAtlas>,
}

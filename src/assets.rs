use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
pub struct EnemiesAssets {
    #[asset(texture_atlas(
        tile_size_x = 16.,
        tile_size_y = 16.,
        columns = 12,
        rows = 3,
        padding_x = 2.,
        padding_y = 2.,
    ))]
    #[asset(path = "images/small_demons.png")]
    pub small_demons: Handle<TextureAtlas>,

    #[asset(texture_atlas(
        tile_size_x = 32.,
        tile_size_y = 32.,
        columns = 4,
        rows = 1,
        padding_x = 22.,
    ))]
    #[asset(path = "images/big_demons.png")]
    pub big_demons: Handle<TextureAtlas>,
}

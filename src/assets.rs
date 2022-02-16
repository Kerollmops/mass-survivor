use bevy::prelude::*;
use bevy_asset_loader::AssetCollection;

#[derive(AssetCollection)]
pub struct AnimatedAssets {
    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 25, rows = 10,))]
    #[asset(path = "images/MagicElementals.png")]
    pub magic_elementals: Handle<TextureAtlas>,

    #[asset(texture_atlas(tile_size_x = 16., tile_size_y = 16., columns = 4, rows = 37,))]
    #[asset(path = "images/Necromancers.png")]
    pub necromancers: Handle<TextureAtlas>,
}

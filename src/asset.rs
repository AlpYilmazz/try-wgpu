use bevy_app::Plugin;
use bevy_asset::{AssetServer, FileAssetIo, AddAsset};
use bevy_ecs::schedule::{SystemStage, StageLabel};

use crate::{CoreStage, Text, TextLoader};


pub struct FlatAssetPlugin;
impl Plugin for FlatAssetPlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let asset_server = create_asset_server();

        app
            .add_stage_after(CoreStage::PreUpdate, AssetStage::LoadAssets, SystemStage::parallel())
            .add_stage_after(CoreStage::PostUpdate, AssetStage::AssetEvents, SystemStage::parallel())
            .insert_resource(asset_server)
            .add_asset::<Text>()
        ;
    }
}

#[derive(StageLabel)]
pub enum AssetStage {
    LoadAssets,
    AssetEvents,
}

fn create_asset_server() -> AssetServer {
    let asset_server = AssetServer::new(FileAssetIo::new(".", false));
    asset_server.add_loader(TextLoader);

    asset_server
}
use client::io::SaveableFormatLoader;
use gameplay::Simulator;
use tracing_subscriber::EnvFilter;

use data::{
    constants::{ASSET_DIRECTORY, LOG_FILTER},
    io::SaveableFormat,
};
use gfx::{AssetLibrary, Gui};

fn main() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(LOG_FILTER.clone()));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::info!("starting application");
    let map = Simulator::temp_create_map();
    let map_data: SaveableFormat = map.try_into().unwrap();
    SaveableFormatLoader::write_saveable(map_data, "res/maps", "testmap").unwrap();

    let map_data = SaveableFormatLoader::read_saveable("./res/maps/testmap.ocmap").unwrap();
    tracing::info!("{map_data}");

    let library = AssetLibrary::load_asset_directory(ASSET_DIRECTORY).unwrap();
    let simulator = Simulator::placeholder([800.0, 600.0]);
    let mut app = Gui::new(library, simulator);
    app.run().unwrap();
}

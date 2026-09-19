use client::io::SaveableFormatLoader;
use gameplay::Simulator;
use tracing_subscriber::EnvFilter;

use data::constants::{ASSET_DIRECTORY, LOG_FILTER};
use gfx::{AssetLibrary, Gui};
use world::map::tile_map::Map;

fn main() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(LOG_FILTER.clone()));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::info!("starting application");
    let map = Simulator::temp_create_map();
    SaveableFormatLoader::write(&map, "res/maps", "testmap").unwrap();
    let map: Map = SaveableFormatLoader::read("./res/maps/testmap.ocmap").unwrap();

    let library = AssetLibrary::load_asset_directory(ASSET_DIRECTORY).unwrap();
    let mut simulator = Simulator::new(map, [800.0, 600.0]);
    simulator.spawn_placeholder_tanks();
    let mut app = Gui::new(library, simulator);
    app.run().unwrap();
}

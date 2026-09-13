use tracing_subscriber::EnvFilter;

use gfx::{AssetLibrary, Gui};
use world::constants::{ASSET_DIRECTORY, DEBUG_PRAGMA};

fn main() {
    let default_level = if cfg!(debug_assertions) {
        DEBUG_PRAGMA
    } else {
        "info"
    };
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::info!("starting application");
    let library = AssetLibrary::load_asset_directory(ASSET_DIRECTORY).unwrap();
    let mut app = Gui::new(library);
    app.run().unwrap();
}

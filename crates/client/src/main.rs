use gameplay::Simulator;
use tracing_subscriber::EnvFilter;

use data::constants::{ASSET_DIRECTORY, LOG_FILTER};
use gfx::{AssetLibrary, Gui};

fn main() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(LOG_FILTER.clone()));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::info!("starting application");
    let library = AssetLibrary::load_asset_directory(ASSET_DIRECTORY).unwrap();
    let simulator = Simulator::placeholder([800.0, 600.0]);
    let mut app = Gui::new(library, simulator);
    app.run().unwrap();
}

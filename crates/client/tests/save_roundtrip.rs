use std::sync::atomic::{AtomicU64, Ordering};

use client::io::SaveableFormatLoader;
use world::Tile;
use world::map::tile_map::Map;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn unique_dir(tag: &str) -> std::path::PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let pid = std::process::id();
    let dir = std::env::temp_dir().join(format!("oc-save-test-{tag}-{pid}-{n}"));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn sample_map() -> Map {
    let tiles = vec![
        Tile::Grass { variant: 0 },
        Tile::Rock,
        Tile::Void,
        Tile::Grass { variant: 2 },
    ];
    Map::new(2, 2, tiles).unwrap()
}

#[test]
fn map_write_then_read_yields_equal_value() {
    let dir = unique_dir("roundtrip");
    let map = sample_map();

    SaveableFormatLoader::write(&map, dir.to_str().unwrap(), "testmap").unwrap();

    let path = dir.join("testmap.ocmap");
    assert!(path.exists(), "file was not created at {path:?}");

    let loaded: Map = SaveableFormatLoader::read(path.to_str().unwrap()).unwrap();
    assert_eq!(map, loaded);
}

#[test]
fn overwriting_existing_file_succeeds() {
    let dir = unique_dir("overwrite");
    let first = sample_map();
    let second = Map::new(1, 1, vec![Tile::Rock]).unwrap();

    SaveableFormatLoader::write(&first, dir.to_str().unwrap(), "map").unwrap();
    SaveableFormatLoader::write(&second, dir.to_str().unwrap(), "map").unwrap();

    let path = dir.join("map.ocmap");
    let loaded: Map = SaveableFormatLoader::read(path.to_str().unwrap()).unwrap();
    assert_eq!(second, loaded);
}

#[test]
fn reading_nonexistent_file_errors() {
    let err =
        SaveableFormatLoader::read::<Map>("/nonexistent/path/does-not-exist.ocmap").unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
}

#[test]
fn reading_corrupted_magic_errors() {
    let dir = unique_dir("corrupt");
    let path = dir.join("bad.ocmap");
    // 8 bogus magic bytes + 3 version bytes + empty body
    std::fs::write(&path, [0u8; 32]).unwrap();

    let err = SaveableFormatLoader::read::<Map>(path.to_str().unwrap()).unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn write_uses_declared_extension() {
    let dir = unique_dir("ext");
    let map = sample_map();
    // Pass a name without extension; loader should append `.ocmap`.
    SaveableFormatLoader::write(&map, dir.to_str().unwrap(), "no_ext").unwrap();
    assert!(dir.join("no_ext.ocmap").exists());
}

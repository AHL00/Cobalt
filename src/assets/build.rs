use std::path::Path;

use walkdir::DirEntry;

/// Assets dir is relative to the project root
/// Output dir is relative to the target directory
pub fn package_assets(relative_assets_dir: &str, output_dir: &str) {
    let target_dir = std::env::var("OUT_DIR").unwrap();

    // Remove everything from the back until the last instance of "target"
    let mut target_dir = target_dir.split("target").collect::<Vec<&str>>()[0].to_string();

    target_dir.push_str("target");
    
    let output_dir = format!("{}/{}", target_dir, output_dir);

    // Create the output directory if it doesn't exist
    if !std::path::Path::new(&output_dir).exists() {
        std::fs::create_dir(&output_dir).unwrap();
    }

    // Clear the output directory
    for entry in std::fs::read_dir(&output_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            std::fs::remove_file(&path).unwrap();
        }
    }

    // Copy all files from the assets directory to the output directory
    let asset_paths: Vec<DirEntry> = walkdir::WalkDir::new(relative_assets_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .collect();

    println!("cargo:warning=Packaging {} assets from \"{}\"", asset_paths.len(), relative_assets_dir);

    println!("cargo:warning=Output directory: \"{}\"", output_dir);

    // let mut assets_hashmap = std::collections::HashMap::new();

    let project_root_str = std::env::var("CARGO_MANIFEST_DIR").unwrap(); 

    let project_root_path = Path::new(&project_root_str);

    let absolute_assets_dir = project_root_path.join(relative_assets_dir).canonicalize().unwrap();
    
    for asset in asset_paths {
        let asset_path = asset.path().canonicalize().unwrap();
        
        let relative_asset_path = super::extract_relative_path(&asset_path, &absolute_assets_dir);

        println!("cargo:rerun-if-changed={}", relative_asset_path);
        println!("cargo:warning=Packaging asset: \"{}\"", relative_asset_path);

    }

    println!("cargo:warning=Finished packaging assets");
}
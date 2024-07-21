use std::{
    default,
    io::BufReader,
    path::{Path, PathBuf},
};

use clap::Parser;

#[derive(clap::Subcommand, Debug)]
enum Command {
    #[clap(name = "pack", long_about = "Add a new asset")]
    Pack {
        handle: String,
        input_file: String,
        #[clap(long, long_help = "Compression level from 0 to 9", default_value("0"))]
        compression: u32,

        #[clap(
            short,
            long,
            long_help = "Path to the output directory. If empty, the working directory is used."
        )]
        output_dir: Option<String>,

        #[clap(
            short,
            long,
            long_help = "Path to the assets directory. If empty, the working directory is used."
        )]
        assets_dir: Option<String>,
    },

    #[clap(name = "remove", long_about = "Remove an existing asset")]
    Remove {
        handle: String,

        #[clap(
            short,
            long,
            long_help = "Path to the assets directory. If empty, the working directory is used."
        )]
        assets_dir: Option<String>,
    },

    #[clap(name = "list", long_about = "List all assets")]
    List {
        #[clap(
            short,
            long,
            long_help = "Path to the assets directory. If empty, the working directory is used."
        )]
        assets_dir: Option<String>,
    },

    #[clap(name = "update", long_about = "Update an existing asset")]
    Update { handle: String, input_file: String },

    #[clap(name = "init", long_about = "Initialize a new assets directory")]
    Init {
        #[clap(
            short,
            long,
            long_help = "Path to the assets directory. If empty, the working directory is used."
        )]
        assets_dir: Option<String>,
    },
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();

    match args.command {
        Command::Pack {
            handle,
            output_dir,
            input_file,
            compression,
            assets_dir,
        } => add(assets_dir, handle, output_dir, input_file, compression)?,

        Command::Remove { handle, assets_dir } => {
            remove(handle, assets_dir)?;
        }

        Command::List { assets_dir } => {
            list(assets_dir)?;
        }

        Command::Update { handle, input_file } => {
            println!(
                "Updating handle: {} with input file: {}",
                handle, input_file
            );
        }

        Command::Init { assets_dir } => {
            init(assets_dir)?;
        }
    }

    Ok(())
}

fn process_assets_path(assets_dir: &PathBuf, path: &str) -> PathBuf {
    let path = Path::new(path);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        assets_dir.join(path)
    }
    .canonicalize()
    .unwrap()
}

fn relative_canonicalize(path: &str) -> PathBuf {
    let path = Path::new(path);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    }
    .canonicalize()
    .map_err(|e| {
        eprintln!("Failed to canonicalize path: {}", path.display());
        e
    })
    .unwrap()
}

fn relative(path: &str) -> PathBuf {
    let path = Path::new(path);

    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().unwrap().join(path)
    }
}

fn add(
    assets_dir: Option<String>,
    handle: String,
    output_dir: Option<String>,
    input_file: String,
    compression: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let assets_dir = relative_canonicalize(&assets_dir.as_ref().unwrap_or(&"./".to_string()));

    let input_file = relative_canonicalize(&input_file);
    let output_file = relative(&output_dir.as_ref().unwrap_or(&"./".to_string()))
        .join(format!("{}.asset", handle));

    if !input_file.exists() {
        eprintln!("Input file does not exist: {}", input_file.display());
        return Ok(());
    }

    if compression > 9 {
        eprintln!("Compression level must be between 0 and 9");
        return Ok(());
    }

    enum AssetType {
        Image,
        GLTF,
    }

    impl AssetType {
        fn str_variants() -> Vec<&'static str> {
            vec!["image", "gltf"]
        }

        fn from_str(s: &str) -> Self {
            match s {
                "image" => AssetType::Image,
                "gltf" => AssetType::GLTF,
                _ => panic!("Invalid asset type"),
            }
        }

        fn pack_asset(
            &self,
            path: &std::path::Path,
            assets_dir: &std::path::Path,
            packed_path: &std::path::Path,
            handle: String,
            compression: Option<u32>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            match self {
                AssetType::Image => {
                    add_image(path, assets_dir, packed_path, handle, compression)?;
                }
           
                AssetType::GLTF => {
                    let gltf_file = std::fs::File::open(path)?;

                    let gltf_reader = BufReader::new(gltf_file);

                    let gltf_res = gltf::Gltf::from_reader(gltf_reader);

                    match gltf_res {
                        Ok(gltf) => {
                            gltf.images().for_each(|image| {
                                println!("Image: {:#?}", image.name());
                            });
                        }

                        Err(e) => {
                            eprintln!("Failed to import GLTF: {:?}", e);
                            return Ok(());
                        }
                    }
                }
            }

            Ok(())
        }
    }

    let asset_type_str =
        inquire::Select::new("Select asset type", AssetType::str_variants()).prompt()?;

    let asset_type = AssetType::from_str(&asset_type_str);

    asset_type.pack_asset(
        &input_file,
        &assets_dir,
        &output_file,
        handle,
        if compression == 0 {
            None
        } else {
            Some(compression)
        },
    )?;

    Ok(())
}

fn add_image(
    path: &std::path::Path,
    assets_dir: &std::path::Path,
    packed_path: &std::path::Path,
    handle: String,
    compression: Option<u32>,
) -> Result<(), Box<dyn std::error::Error>> {
    enum TextureType {
        RGBA32Float,
        RGBA16Float,
        RGBA8Unorm,
        RGBA8UnormSrgb,
        R32Float,
        R16Float,
        R8Unorm,
        R8Uint,
        R8Snorm,
    }

    impl TextureType {
        fn str_variants() -> Vec<&'static str> {
            vec![
                "rgba32float",
                "rgba16float",
                "rgba8unorm",
                "rgba8unormsrgb",
                "r32float",
                "r16float",
                "r8unorm",
                "r8uint",
                "r8snorm",
            ]
        }

        fn from_str(s: &str) -> Self {
            match s {
                "rgba32float" => TextureType::RGBA32Float,
                "rgba16float" => TextureType::RGBA16Float,
                "rgba8unorm" => TextureType::RGBA8Unorm,
                "rgba8unormsrgb" => TextureType::RGBA8UnormSrgb,
                "r32float" => TextureType::R32Float,
                "r16float" => TextureType::R16Float,
                "r8unorm" => TextureType::R8Unorm,
                "r8uint" => TextureType::R8Uint,
                "r8snorm" => TextureType::R8Snorm,
                _ => panic!("Invalid texture type"),
            }
        }

        fn pack_asset(
            &self,
            path: &std::path::Path,
            assets_dir: &std::path::Path,
            packed_path: &std::path::Path,
            handle: String,
            compression: Option<u32>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            match self {
                TextureType::RGBA32Float => {
                    let reader = BufReader::new(std::fs::File::open(path)?);
                    let asset_data = TextureAsset::<
                        {
                            cobalt_core::graphics::texture::TextureType::RGBA32Float
                        },
                    >::read_from_file_to_buffer(
                        reader, path
                    )?;

                    cobalt_core::assets::pack::pack_asset::<
                        TextureAsset<
                            {
                                cobalt_core::graphics::texture::TextureType::RGBA32Float
                            },
                        >,
                    >(
                        asset_data, assets_dir, packed_path, handle, compression
                    )?;

                    Ok(())
                }

                TextureType::RGBA8Unorm => {
                    let reader = BufReader::new(std::fs::File::open(path)?);
                    let asset_data = TextureAsset::<
                        { cobalt_core::graphics::texture::TextureType::RGBA8Unorm },
                    >::read_from_file_to_buffer(
                        reader, path
                    )?;

                    cobalt_core::assets::pack::pack_asset::<
                        TextureAsset<
                            {
                                cobalt_core::graphics::texture::TextureType::RGBA8Unorm
                            },
                        >,
                    >(
                        asset_data, assets_dir, packed_path, handle, compression
                    )?;

                    Ok(())
                }

                _ => {
                    eprintln!("Unsupported texture type");
                    Ok(())
                }
            }
        }
    }

    let texture_type_str =
        inquire::Select::new("Select texture type", TextureType::str_variants())
            .prompt()?;

    let texture_type = TextureType::from_str(&texture_type_str);

    texture_type.pack_asset(path, assets_dir, packed_path, handle, compression)?;

    Ok(())
}

fn remove(handle: String, assets_dir: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let assets_dir = relative_canonicalize(assets_dir.as_ref().unwrap_or(&".".to_string()));

    let manifest = cobalt_core::assets::pack::Manifest::load(std::path::Path::new(&assets_dir));

    match manifest {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to read manifest: {:?}", e);
            return Ok(());
        }
    }

    let mut manifest = manifest.unwrap();

    let asset = manifest.assets.iter().find(|a| a.handle == handle);

    match asset {
        Some(asset) => {
            println!("Deleting asset: {}", &asset.packed_file);
            std::fs::remove_file(&asset.packed_file)?;
        }
        None => {
            eprintln!("Asset not found: {}", handle);
            return Ok(());
        }
    }


    println!("Removing handle in manifest: {}", handle);

    manifest.assets.retain(|a| a.handle != handle);

    let manifest_path = assets_dir.join("manifest.toml");

    let manifest = toml::to_string(&manifest)?;

    std::fs::write(manifest_path, manifest)?;

    Ok(())
}

use cli_table::{format::Justify, print_stdout, Table, WithTitle};
use cobalt_core::{
    assets::asset::AssetTrait, exports::assets::TextureAsset, gltf, graphics::texture::TextureType, utils::bytes
};

#[derive(Table)]
struct ListEntry {
    #[table(title = "Handle")]
    handle: String,
    #[table(title = "Type")]
    type_name: String,
    #[table(title = "File Size")]
    size: u64,
    #[table(title = "Last Modified")]
    modified: String,
}

fn list(assets_dir: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = Vec::new();

    let assets_dir = relative_canonicalize(assets_dir.as_ref().unwrap_or(&".".to_string()));

    let manifest = cobalt_core::assets::pack::Manifest::load(std::path::Path::new(&assets_dir));

    match manifest {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to read manifest: {:?}", e);
            return Ok(());
        }
    }

    let manifest = manifest.unwrap();

    for asset in manifest.assets {
        let metadata = std::fs::metadata(&asset.packed_file)?;
        let modified = metadata.modified()?;

        entries.push(ListEntry {
            handle: asset.handle,
            size: metadata.len(),
            type_name: asset.type_name,
            modified: humantime::Timestamp::from(modified).to_string(),
        });
    }

    cli_table::print_stdout(entries.with_title())?;

    Ok(())
}

fn init(assets_dir: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let assets_dir = relative_canonicalize(&assets_dir.as_ref().unwrap_or(&".".to_string()));

    let assets_dir = std::path::Path::new(&assets_dir);

    if !assets_dir.exists() {
        std::fs::create_dir_all(assets_dir)?;
    }

    let manifest_path = assets_dir.join("manifest.toml");

    if !manifest_path.exists() {
        let manifest = cobalt_core::assets::pack::Manifest { assets: Vec::new() };

        let manifest = toml::to_string(&manifest)?;

        std::fs::write(manifest_path, manifest)?;
        println!("Initialized assets directory at: {}", assets_dir.display());
    } else {
        eprintln!("Manifest already exists at: {}", manifest_path.display());
    }

    Ok(())
}

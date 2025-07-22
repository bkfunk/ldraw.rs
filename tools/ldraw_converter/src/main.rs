/// CLI tool for converting LDraw files to a different format.
/// This tool requires the LDraw directory to be specified either via the `--ldraw-dir` option or the `LDRAWDIR` environment variable.
/// It processes LDraw files and outputs them in a specified format.
/// It can convert to:
/// - JSON
/// - Rust objects
/// - Other formats as specified by the user.
///
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use clap::{Arg, Command};
use ldraw::{
    color::ColorCatalog,
    library::{CacheCollectionStrategy, PartCache, resolve_dependencies_multipart},
    parser::{parse_color_definitions, parse_multipart_document},
    resolvers::local::LocalLoader,
};
use ldraw_ir::part::{Part, bake_part_with_couplings};
use std::sync::{Arc, RwLock};
use tokio::{fs::File, io::BufReader};

#[tokio::main]
async fn main() {
    let matches = Command::new("ldraw_converter")
        .about("Convert LDraw files to different formats")
        .arg(
            Arg::new("ldraw_dir")
                .long("ldraw-dir")
                .value_name("PATH")
                .help("Path to LDraw directory"),
        )
        .arg(
            Arg::new("output_format")
                .short('f')
                .long("format")
                .default_value("json")
                .help("Output format (e.g., json, rust)"),
        )
        .arg(
            Arg::new("input_files")
                .num_args(1..)
                .required(true)
                .help("Input LDraw files to convert"),
        )
        .arg(
            Arg::new("output_dir")
                .short('o')
                .long("output-dir")
                .value_name("PATH")
                .help("Output directory for converted files"),
        )
        .get_matches();

    // Get LDraw directory
    let ldraw_dir = match matches.get_one::<String>("ldraw_dir") {
        Some(v) => v.to_string(),
        None => match env::var("LDRAWDIR") {
            Ok(v) => v,
            Err(_) => {
                eprintln!(
                    "Error: --ldraw-dir option or LDRAWDIR environment variable is required."
                );
                std::process::exit(1);
            }
        },
    };

    let output_format = matches
        .get_one::<String>("output_format")
        .map(|s| s.as_str())
        .unwrap_or("json");
    let output_dir = matches.get_one::<String>("output_dir").map(|s| s.as_str());

    // Initialize LDraw loader and color catalog
    let ldraw_path = PathBuf::from(&ldraw_dir);
    let loader = LocalLoader::new(Some(ldraw_path.clone()), None);

    let colors = match parse_color_definitions(&mut BufReader::new(
        File::open(ldraw_path.join("LDConfig.ldr"))
            .await
            .expect("Could not load color definition file"),
    ))
    .await
    {
        Ok(colors) => colors,
        Err(e) => {
            eprintln!("Error parsing color definitions: {}", e);
            std::process::exit(1);
        }
    };

    let cache = Arc::new(RwLock::new(PartCache::new()));

    // Process input files
    if let Some(input_files) = matches.get_many::<String>("input_files") {
        for file_path in input_files {
            let path = Path::new(file_path);
            if let Err(e) = process_ldraw_file(
                path,
                &loader,
                &colors,
                Arc::clone(&cache),
                output_format,
                output_dir,
            )
            .await
            {
                eprintln!("Error processing {}: {}", file_path, e);
            }
        }
    }

    let collected = cache
        .write()
        .expect("Failed to acquire write lock on PartCache (RwLock may be poisoned)")
        .collect(CacheCollectionStrategy::PartsAndPrimitives);
    println!("Processed {} parts total.", collected);
}

async fn process_ldraw_file<L: ldraw::library::LibraryLoader>(
    file_path: &Path,
    loader: &L,
    colors: &ColorCatalog,
    cache: Arc<RwLock<PartCache>>,
    output_format: &str,
    output_dir: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Processing: {}", file_path.display());

    // Parse the LDraw file
    let file = File::open(file_path).await?;
    let document = parse_multipart_document(&mut BufReader::new(file), colors).await?;

    // Resolve dependencies
    let resolution_result = resolve_dependencies_multipart(
        &document,
        Arc::clone(&cache),
        colors,
        loader,
        &|alias, result| {
            if let Err(err) = result {
                eprintln!("Could not resolve dependency {}: {}", alias, err);
            }
        },
    )
    .await;

    // Create the part with integrated coupling detection
    let part = tokio::task::spawn_blocking(move || {
        bake_part_with_couplings(&document, &resolution_result, false)
    })
    .await?;

    // Serialize and save
    let output_path = get_output_path(file_path, output_format, output_dir)?;
    save_part(&part, &output_path, output_format).await?;

    println!(
        "Saved: {} ({} couplings detected)",
        output_path.display(),
        part.couplings.len()
    );
    Ok(())
}

// Generate output file path
fn get_output_path(
    input_path: &Path,
    output_format: &str,
    output_dir: Option<&str>,
) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let file_stem = input_path.file_stem().ok_or("Invalid input file path")?;

    let extension = match output_format {
        "json" => "json",
        "rust" => "bin",
        _ => return Err(format!("Unsupported output format: {}", output_format).into()),
    };

    let output_path = if let Some(dir) = output_dir {
        let dir_path = Path::new(dir);
        fs::create_dir_all(dir_path)?;
        dir_path.join(format!("{}.{}", file_stem.to_string_lossy(), extension))
    } else {
        input_path.with_extension(extension)
    };

    Ok(output_path)
}

// Save the part to file
async fn save_part(
    part: &Part,
    output_path: &Path,
    output_format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    match output_format {
        "json" => {
            let json_data = serde_json::to_string_pretty(part)?;
            fs::write(output_path, json_data)?;
        }
        "rust" => {
            // For Rust format, we'll serialize as bincode
            let bincode_data = bincode::serialize(part)?;
            fs::write(output_path, bincode_data)?;
        }
        _ => return Err(format!("Unsupported output format: {}", output_format).into()),
    }
    Ok(())
}

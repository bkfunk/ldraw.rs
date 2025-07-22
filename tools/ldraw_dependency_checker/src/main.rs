/// CLI tool for checking LDraw library dependencies.
/// This tool requires the LDraw directory to be specified either via the `--ldraw-dir` option or the `LDRAWDIR` environment variable.
/// It analyzes LDraw files to find missing dependencies and can suggest copy commands to resolve them.

use std::{
    collections::HashSet,
    env,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use clap::{Arg, Command};
use ldraw::{
    library::{resolve_dependencies_multipart, PartCache},
    parser::{parse_color_definitions, parse_multipart_document},
    resolvers::local::LocalLoader,
};
use tokio::{fs::File, io::BufReader};
use walkdir::WalkDir;

/// Normalize an LDraw path to use OS-specific path separators
fn normalize_ldraw_path(ldraw_path: &str) -> PathBuf {
    let components: Vec<&str> = ldraw_path
        .split('/')
        .flat_map(|part| part.split('\\'))
        .filter(|part| !part.is_empty())
        .collect();
    components.iter().collect()
}

/// Generate platform-specific copy commands for missing files
fn generate_copy_command(dep: &str, source_lib: &Path, target_lib: &Path) {
    let normalized_dep = normalize_ldraw_path(dep);
    let dep_str = normalized_dep.to_string_lossy();

    #[cfg(unix)]
    {
        println!("  # Try these locations:");
        println!(
            "  cp {}/parts/{} {}/parts/",
            source_lib.display(), dep_str, target_lib.display()
        );
        println!(
            "  cp {}/parts/s/{} {}/parts/s/",
            source_lib.display(), dep_str, target_lib.display()
        );
        println!(
            "  cp {}/p/{} {}/p/",
            source_lib.display(), dep_str, target_lib.display()
        );
    }

    #[cfg(windows)]
    {
        println!("  REM Try these locations:");
        println!(
            "  copy \"{}\\parts\\{}\" \"{}\\parts\\\"",
            source_lib.display(), dep_str, target_lib.display()
        );
        println!(
            "  copy \"{}\\parts\\s\\{}\" \"{}\\parts\\s\\\"",
            source_lib.display(), dep_str, target_lib.display()
        );
        println!(
            "  copy \"{}\\p\\{}\" \"{}\\p\\\"",
            source_lib.display(), dep_str, target_lib.display()
        );
    }

    #[cfg(not(any(unix, windows)))]
    {
        println!("  # Copy {} from {} to {}", dep_str, source_lib.display(), target_lib.display());
        println!("  # Check: parts/, parts/s/, or p/ subdirectories");
    }
}

async fn load_part_document(path: &Path) -> Result<ldraw::document::MultipartDocument, Box<dyn std::error::Error>> {
    let file = File::open(path).await?;
    let mut reader = BufReader::new(file);
    let colors = std::collections::HashMap::new(); // Empty color catalog for parsing
    
    parse_multipart_document(&mut reader, &colors)
        .await
        .map_err(|e| format!("Failed to parse LDraw file {}: {}", path.display(), e).into())
}

async fn find_all_part_files(library_path: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut part_files = Vec::new();

    for entry in WalkDir::new(library_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some(std::ffi::OsStr::new("dat")))
    {
        part_files.push(entry.path().to_path_buf());
    }

    Ok(part_files)
}

#[tokio::main]
async fn main() {
    let matches = Command::new("ldraw_dependency_checker")
        .about("Check LDraw library for missing dependencies")
        .arg(
            Arg::new("ldraw_dir")
                .long("ldraw-dir")
                .value_name("PATH")
                .help("Path to LDraw directory for dependency resolution"),
        )
        .arg(
            Arg::new("library_path")
                .short('l')
                .long("library")
                .value_name("PATH")
                .required(true)
                .help("Path to the LDraw library directory to check"),
        )
        .arg(
            Arg::new("parts")
                .short('p')
                .long("parts")
                .value_delimiter(',')
                .num_args(1..)
                .help("Check specific part files instead of entire library"),
        )
        .arg(
            Arg::new("show_all")
                .short('a')
                .long("show-all")
                .action(clap::ArgAction::SetTrue)
                .help("Show all dependencies (not just missing ones)"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Verbose output"),
        )
        .get_matches();

    // Get LDraw directory for dependency resolution
    let ldraw_dir = match matches.get_one::<String>("ldraw_dir") {
        Some(v) => v.to_string(),
        None => match env::var("LDRAWDIR") {
            Ok(v) => v,
            Err(_) => {
                eprintln!(
                    "Error: --ldraw-dir option or LDRAWDIR environment variable is required for dependency resolution."
                );
                std::process::exit(1);
            }
        },
    };

    let library_path = PathBuf::from(
        matches
            .get_one::<String>("library_path")
            .expect("Library path is required"),
    );

    if !library_path.exists() {
        eprintln!("Error: Library path does not exist: {}", library_path.display());
        std::process::exit(1);
    }

    let verbose = matches.get_flag("verbose");
    let show_all = matches.get_flag("show_all");

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
            eprintln!("Warning: Could not parse color definitions: {}", e);
            eprintln!("Continuing with empty color catalog...");
            std::collections::HashMap::new()
        }
    };

    let cache = Arc::new(RwLock::new(PartCache::new()));

    // Determine which files to check
    let part_files = if let Some(parts) = matches.get_many::<String>("parts") {
        // Check specific parts
        let mut files = Vec::new();
        for part in parts {
            let part_path = library_path.join("parts").join(part);
            if part_path.exists() {
                files.push(part_path);
            } else {
                eprintln!("Part not found: {}", part_path.display());
            }
        }
        files
    } else {
        // Check entire library
        match find_all_part_files(&library_path).await {
            Ok(files) => files,
            Err(e) => {
                eprintln!("Error finding part files: {}", e);
                std::process::exit(1);
            }
        }
    };

    if part_files.is_empty() {
        println!("No part files found to check.");
        return;
    }

    println!("Checking {} part files...", part_files.len());

    let all_missing = Arc::new(RwLock::new(HashSet::new()));
    let all_found = Arc::new(RwLock::new(HashSet::new()));
    let mut processed_parts = 0;

    for part_path in part_files {
        if verbose {
            println!("Checking: {}", part_path.display());
        }

        match load_part_document(&part_path).await {
            Ok(document) => {
                processed_parts += 1;

                // Clone the Arc references for the closure
                let missing_clone = Arc::clone(&all_missing);
                let found_clone = Arc::clone(&all_found);

                // Resolve dependencies for this document
                let result = resolve_dependencies_multipart(
                    &document,
                    Arc::clone(&cache),
                    &colors,
                    &loader,
                    &|alias, result| match result {
                        Ok(_) => {
                            found_clone.write().unwrap().insert(alias.to_string());
                        }
                        Err(ldraw::error::ResolutionError::FileNotFound) => {
                            missing_clone.write().unwrap().insert(alias.to_string());
                        }
                        Err(e) => {
                            eprintln!("Resolution error for {}: {}", alias, e);
                            missing_clone.write().unwrap().insert(alias.to_string());
                        }
                    },
                )
                .await;

                // Also track the dependencies from the result
                for dependency in result.list_dependencies() {
                    all_found.write().unwrap().insert(dependency.to_string());
                }

                if verbose {
                    println!(
                        "  -> Found {} dependencies",
                        result.list_dependencies().len()
                    );
                }
            }
            Err(e) => {
                eprintln!("Failed to load {}: {}", part_path.display(), e);
            }
        }
    }

    // Extract final results from Arc<RwLock<HashSet>>
    let all_missing = all_missing.read().unwrap().clone();
    let all_found = all_found.read().unwrap().clone();

    println!("\n=== DEPENDENCY ANALYSIS RESULTS ===");
    println!("Processed {} part files", processed_parts);

    if show_all && !all_found.is_empty() {
        println!("\nFound dependencies ({}):", all_found.len());
        let mut found_vec: Vec<_> = all_found.iter().collect();
        found_vec.sort();
        for dep in found_vec {
            println!("  ✓ {}", dep);
        }
    }

    if !all_missing.is_empty() {
        println!("\nMissing dependencies ({}):", all_missing.len());
        let mut missing_vec: Vec<_> = all_missing.iter().collect();
        missing_vec.sort();
        for dep in &missing_vec {
            println!("  ✗ {}", dep);
        }

        println!("\nTo copy missing files from source library:");
        for dep in &missing_vec {
            generate_copy_command(dep, &ldraw_path, &library_path);
        }
    } else {
        println!("\n✅ All dependencies resolved successfully!");
    }
}

use crate::cli::{Cli, Commands};
use crate::core::{add_resource, export_resource, find_resources_config, remove_resource};
use clap::Parser;
use std::process::ExitCode;

mod cli;
mod core;
mod util;

#[cfg(test)]
mod tests;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        // 列出资源
        Commands::List { target_file, id } => {
            println!("Listing resources from \"{}\":", target_file.display());
            match find_resources_config(&target_file, |_pos, _config| ()) {
                Ok(configs) => {
                    let filtered: Vec<_> = if let Some(ref filter_id) = id {
                        configs
                            .iter()
                            .filter(|c| c.id().trim() == filter_id.trim())
                            .collect()
                    } else {
                        configs.iter().collect()
                    };

                    let count = filtered.len();
                    for config in filtered {
                        println!(
                            "  ID: {} | Name: {} | Size: {} bytes | Compressed: {}",
                            config.id().trim(),
                            config.name().trim(),
                            config.size().trim().parse().unwrap_or(0),
                            if config.compress() == core::CompressMode::Compress {
                                "Yes"
                            } else {
                                "No"
                            }
                        );
                    }

                    println!("Found {} resource(s)", count);
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Failed to list resources: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        // 增加资源
        Commands::Add {
            target_file,
            resources,
            id,
            new_file_path,
            compression,
        } => {
            println!(
                "Adding resource \"{}\" (ID: {}) to \"{}\"...",
                resources.display(),
                id,
                target_file.display()
            );
            match add_resource(
                &target_file,
                &resources,
                &id,
                if compression == 0 {
                    None
                } else {
                    Some(compression)
                },
                new_file_path.as_deref(),
            ) {
                Ok(()) => {
                    println!("Resource added successfully");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Failed to add resource: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        // 导出资源
        Commands::Export {
            target_file,
            id,
            output_path,
        } => {
            println!(
                "Exporting resource (ID: {}) from \"{}\" to \"{}\"...",
                id,
                target_file.display(),
                output_path.display()
            );
            match export_resource(&target_file, &id, &output_path) {
                Ok(()) => {
                    println!("Resource exported successfully");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Failed to export resource: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
        // 删除资源
        Commands::Remove {
            target_file,
            id,
            new_file_path,
        } => {
            println!(
                "Removing resource (ID: {}) from \"{}\"...",
                id,
                target_file.display()
            );
            match remove_resource(&target_file, &id, new_file_path.as_deref()) {
                Ok(()) => {
                    println!("Resource removed successfully");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("Failed to remove resource: {}", e);
                    ExitCode::FAILURE
                }
            }
        }
    }
}

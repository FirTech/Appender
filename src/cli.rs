use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all resources in a file
    List {
        /// Target file path
        #[arg(value_parser = validate_path)]
        target_file: PathBuf,
        /// Resource ID to filter (optional)
        #[arg(short, long)]
        id: Option<String>,
    },
    /// Add resources
    Add {
        /// Target file path
        #[arg(value_parser = validate_path)]
        target_file: PathBuf,
        /// Resource file path
        #[arg(value_parser = validate_path)]
        resources: PathBuf,
        /// Resource ID
        id: String,
        /// New file path (optional)
        new_file_path: Option<PathBuf>,
        /// Compression level (0-9)
        #[arg(short, long, default_value = "1")]
        compression: u32,
    },
    /// Export resources
    Export {
        /// Target file path
        #[arg(value_parser = validate_path)]
        target_file: PathBuf,
        /// Resource ID
        id: String,
        /// Output path
        output_path: PathBuf,
    },
    /// Remove a resource by ID
    Remove {
        /// Target file path
        #[arg(value_parser = validate_path)]
        target_file: PathBuf,
        /// Resource ID
        id: String,
        /// New file path (optional)
        new_file_path: Option<PathBuf>,
    },
}

/// 验证路径是否存在
///
/// # 参数
/// - `s`: 路径字符串
///
/// # 返回值
/// - `Ok(PathBuf)`: 路径存在
/// - `Err(err)`: 路径不存在
fn validate_path(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if !path.exists() {
        return Err(
            "The path does not exist, please make sure the entered directory exists".to_string(),
        );
    }
    Ok(path)
}

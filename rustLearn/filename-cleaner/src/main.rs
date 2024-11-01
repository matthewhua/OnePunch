use std::fs;
use std::path::Path;
use regex::Regex;
use walkdir::WalkDir;
use indicatif::{ProgressBar, ProgressStyle};
use log::{info, warn, error};
use simple_logger::SimpleLogger;
use clap::Parser;

#[derive(Parser)]
#[clap(version = "1.0", author = "Matthew Hua")]
struct Opts {
    /// Directory to process
    #[clap(default_value = ".")]
    dir: String,

    /// Whether to show preview only
    #[clap(short, long)]
    dry_run: bool,

    /// Whether to show verbose output
    #[clap(short, long)]
    verbose: bool,
}

fn main() -> std::io::Result<()> {
    // 解析命令行参数
    let opts = Opts::parse();

    // 初始化日志
    SimpleLogger::new()
        .with_level(if opts.verbose { log::LevelFilter::Debug } else { log::LevelFilter::Info })
        .init()
        .unwrap();

    let dir_path = Path::new(&opts.dir);
    let re = Regex::new(r"【.*?】").unwrap();

    info!("Processing directory: {}", dir_path.display());

    // 计算总文件数并创建进度条
    let total_files = WalkDir::new(dir_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();

    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(ProgressStyle::default_bar()
        .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
        .unwrap());

    let mut count = 0;

    for entry in WalkDir::new(dir_path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.contains("【") && file_name.contains("】") {
                    let new_name = re.replace_all(file_name, "").to_string();
                    let new_path = entry.path().with_file_name(&new_name);

                    info!("Renaming: {} -> {}", file_name, new_name);

                    if !opts.dry_run {
                        match fs::rename(entry.path(), &new_path) {
                            Ok(_) => count += 1,
                            Err(e) => error!("Failed to rename {}: {}", file_name, e),
                        }
                    } else {
                        count += 1;
                    }
                }
            }
        }
        pb.inc(1);
    }

    pb.finish_with_message("done");

    info!("Successfully processed {} files", count);
    if opts.dry_run {
        info!("This was a dry run. No files were actually renamed.");
    }

    Ok(())
}
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use easytube_core::provider::{BinaryProvider, ProviderType};
use easytube_core::state::{DownloadManager, DownloadSettings};

#[derive(Parser)]
#[command(name = "easytube", about = "Simple video downloader powered by yt-dlp")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Download a video
    #[command(alias = "dl")]
    Download {
        /// Video URL
        url: String,

        /// Format ID (e.g., "mp4-720p", "best", leave empty for best quality)
        #[arg(short, long)]
        format: Option<String>,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Probe a video URL and print metadata
    Probe {
        /// Video URL
        url: String,
    },

    /// List download history
    History {
        /// Limit the number of entries
        #[arg(long, default_value = "20")]
        limit: usize,
    },

    /// Get or set settings
    Settings {
        /// Setting key (e.g., "clipboard_monitor", "max_concurrent")
        key: Option<String>,

        /// New value (omit to read current value)
        value: Option<String>,
    },
}

pub fn run(args: &[String]) {
    let ytdlp = BinaryProvider::find_ytdlp(&ProviderType::System)
        .or_else(|| BinaryProvider::find_ytdlp(&ProviderType::Downloaded))
        .unwrap_or_else(|| PathBuf::from("yt-dlp"));

    let ffmpeg = BinaryProvider::find_ffmpeg(&ProviderType::System)
        .or_else(|| BinaryProvider::find_ffmpeg(&ProviderType::Downloaded));

    let manager = DownloadManager::new(DownloadSettings::default(), ytdlp, ffmpeg);

    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");

    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Download { url, format, output } => {
            rt.block_on(async {
                match manager.create_download(url.clone(), format.clone(), output).await {
                    Ok(job_id) => {
                        eprintln!("Download started. job_id: {}", job_id);
                        eprintln!("URL: {}", url);

                        let out_dir = manager.output_dir().clone()
                            .to_string_lossy()
                            .to_string();
                        let fmt = format.as_deref().unwrap_or("best");

                        let status = tokio::process::Command::new(manager.ytdlp_path())
                            .args([
                                "-o",
                                &format!("{}/%(title)s.%(ext)s", out_dir),
                                "-f",
                                fmt,
                                "--no-playlist",
                                &url,
                            ])
                            .status()
                            .await;

                        match status {
                            Ok(s) if s.success() => {
                                eprintln!("Download complete.");
                            }
                            Ok(s) => {
                                eprintln!("Download failed with exit code: {:?}", s.code());
                                std::process::exit(1);
                            }
                            Err(e) => {
                                eprintln!("Download error: {}", e);
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error creating download: {}", e);
                        std::process::exit(1);
                    }
                }
            });
        }

        Commands::Probe { url } => {
            rt.block_on(async {
                match manager.probe(&url).await {
                    Ok(info) => {
                        println!("Title: {}", info.title);
                        if let Some(dur) = &info.duration {
                            println!("Duration: {}", dur);
                        }
                        println!("Formats: {}", info.formats.len());
                        for f in &info.formats.iter().take(20).collect::<Vec<_>>() {
                            println!(
                                "  {}  {:8}  {:>10}  {}",
                                f.id,
                                f.ext,
                                f.resolution.as_deref().unwrap_or("N/A"),
                                f.note.as_deref().unwrap_or(""),
                            );
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            });
        }

        Commands::History { limit } => {
            eprintln!("History is managed by the GUI (IndexedDB).");
            eprintln!("Use the desktop app to view download history.");
            let _ = limit;
        }

        Commands::Settings { key, value } => {
            match (key, value) {
                (Some(k), Some(v)) => {
                    println!("Setting {} = {}", k, v);
                    eprintln!("Settings persistence not yet implemented.");
                }
                (Some(k), None) => {
                    println!("Reading setting: {}", k);
                    eprintln!("Settings persistence not yet implemented.");
                }
                (None, _) => {
                    println!("max_concurrent: 5");
                    println!("language: zh-TW");
                }
            }
        }
    }
}

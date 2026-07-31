// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "mcp" => {
                easytube_mcp::serve();
            }
            "download" | "probe" | "history" | "settings" => {
                easytube_cli::run(&args[1..]);
            }
            _ => {
                easytube_lib::run();
            }
        }
    } else {
        easytube_lib::run();
    }
}

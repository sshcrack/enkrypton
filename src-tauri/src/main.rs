// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{fs::File, io::Write, env::current_exe};

//const TOR_BIN: &'static [u8;7803392]  = ;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() {
    let tor_exe_raw = include_bytes!("./assets/tor.exe");
    let mut tor_write_path = current_exe().unwrap();
    tor_write_path.set_file_name("tor_proxy.exe");


    println!("Writing at {:?}", tor_write_path);
    let mut f = File::create(tor_write_path).unwrap();
    f.write_all(tor_exe_raw).unwrap();
    drop(f);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod bridge;
mod clipboard;
mod event;
mod notes;
mod storage;

use std::collections::HashMap;
use tokio::sync::Mutex;

use clipboard::Clipboard;

use bridge::bridge_manager::BRIDGE_MANAGER;
use event::event_manager::EVENT_MANAGER;
use notes::{Document, Note, NoteManager};
use storage::Storage;

use tauri::Manager;

use log::LevelFilter;

use hyprland::data::Monitor;
use hyprland::keyword::Keyword;
use hyprland::shared::HyprDataActive;

fn add_hypr_position_rule() {
    let monitor = Monitor::get_active().unwrap();

    let (w, h) = (300, 600);

    let (m_w, m_h) = (monitor.width, monitor.height);

    let (to_x, to_y) = (m_w as i16 - w, m_h as i16 - h);

    // windowrulev2 = blur:off, class:^(kitty)$, focus:off
    Keyword::set(
        "windowrulev2",
        format!("blur:off , class:^(tools-desktop-tauri), focus:off"),
    )
    .unwrap();
    Keyword::set(
        "windowrulev2",
        format!("float , class:^(tools-desktop-tauri)"),
    )
    .unwrap();
    Keyword::set(
        "windowrulev2",
        format!("move {to_x} {to_y}, class:^(tools-desktop-tauri)"),
    )
    .unwrap();
    // Keyword::set(
    //     "windowrulev2",
    //     format!("size 300 600, class:^(tools-desktop-tauri)"),
    // )
    // .unwrap();
}

struct AppState {
    note_manager: Mutex<NoteManager>,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn get_advertisement() -> String {
    println!("Getting advert");
    let bridge_manager = BRIDGE_MANAGER.clone();
    bridge_manager.get_advertisement().await
}

#[tauri::command]
async fn get_notes(
    note_manager: tauri::State<'_, Mutex<NoteManager>>,
) -> Result<HashMap<String, Note>, ()> {
    let notes = note_manager.lock().await.get_all().await;
    println!("Notes : {notes:?}");
    Ok(notes)
}
#[tauri::command]
async fn get_note(
    note_manager: tauri::State<'_, Mutex<NoteManager>>,
    id: String,
) -> Result<Note, ()> {
    Ok(note_manager.lock().await.get(&id).await.unwrap())
}

#[tauri::command]
async fn new_note(
    note_manager: tauri::State<'_, Mutex<NoteManager>>,
    doc: Option<Document>,
) -> Result<Note, ()> {
    Ok(note_manager.lock().await.create(doc).await.unwrap())
}

#[tauri::command]
async fn update_note(
    note_manager: tauri::State<'_, Mutex<NoteManager>>,
    id: String,
    doc: Document,
) -> Result<Note, ()> {
    Ok(note_manager.lock().await.update(&id, doc).await.unwrap())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    add_hypr_position_rule();

    // Initialize the storage
    Storage::init();

    // Create and start the event manager
    let event_manager = EVENT_MANAGER.clone();
    event_manager.run();

    // Create and start the clipboard manager
    let clipboard_manager = Clipboard::new();
    clipboard_manager.run();

    // Create and start the bridge manager
    let bridge_manager = BRIDGE_MANAGER.clone();
    tauri::async_runtime::spawn(async move {
        bridge_manager.setup().await;
    });

    // event_manager
    //     .on({
    //         let router = router.clone();
    //         move |_: ConnectedEvent| {
    //             let router = router.clone();
    //             async move {
    //                 let router = router.clone();
    //                 router
    //                     .navigate::<BridgesController>(BridgesScreenData {})
    //                     .await;
    //             }
    //         }
    //     })
    //     .await;

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_advertisement,
            get_notes,
            new_note,
            update_note,
            get_note
        ])
        .setup(|app| {
            app.manage(Mutex::new(NoteManager::new()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

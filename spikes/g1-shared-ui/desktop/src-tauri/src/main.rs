fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("G1 Tauri fixture failed")
}

// POC commands for ui-next testing

#[tauri::command]
pub fn greet(name: String) -> String {
    format!("Hello, {}! You've successfully called Tauri from Svelte 5.", name)
}

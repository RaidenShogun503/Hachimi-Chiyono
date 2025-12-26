use crate::core::Hachimi;

#[derive(serde::Deserialize, serde::Serialize, Clone, Default)]
pub struct Config {
    // iOS specific config fields if any
}

pub fn is_il2cpp_lib(filename: &str) -> bool {
    filename.contains("UnityFramework") // iOS usually has UnityFramework.
}

pub fn is_criware_lib(_filename: &str) -> bool {
    false
}

pub fn on_hooking_finished(hachimi: &Hachimi) {
    log::info!("iOS Hooking finished!");
}

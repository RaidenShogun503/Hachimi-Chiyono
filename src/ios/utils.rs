
use std::path::PathBuf;

pub fn get_internal_storage() -> PathBuf {
    // On iOS, we are sandboxed. Documents directory is usually a good place.
    // We can get HOME env var.
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join("Documents");
    }
    PathBuf::from(".")
}

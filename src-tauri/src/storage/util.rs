use std::path::Path;

use crate::util::get_root_dir;

use super::{StorageManager, STORAGE};

pub fn get_storage_path() -> Box<Path> {
    let mut root = get_root_dir();
    root.push("storage.bin");

    return root.into_boxed_path();
}
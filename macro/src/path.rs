use crate::error::*;
use std::path::Path;

pub fn parent(path: &Path) -> Result<&Path> {
    path.parent().context(|| error!("Path '{}' does not have a parent.", path.display()))
}

pub fn find_parent<'t>(path: &'t Path, dir_name: &str) -> Result<&'t Path> {
    let dir_name_os = std::ffi::OsStr::new(dir_name);
    path.ancestors()
        .find(|p| p.file_name() == Some(dir_name_os))
        .context(|| error!(
            "Path '{}' does not have parent '{dir_name}' directory.",
            path.display()
        ))
}
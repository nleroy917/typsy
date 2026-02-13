use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::error::{Result, TypsyError};

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(src).unwrap();
        let dst_path = dst.join(rel_path);

        if src_path.is_dir() {
            fs::create_dir_all(&dst_path).map_err(|e| TypsyError::Io {
                path: dst_path.clone(),
                source: e,
            })?;
        } else {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent).map_err(|e| TypsyError::Io {
                    path: parent.to_path_buf(),
                    source: e,
                })?;
            }
            fs::copy(src_path, &dst_path).map_err(|e| TypsyError::Io {
                path: dst_path.clone(),
                source: e,
            })?;
        }
    }
    Ok(())
}

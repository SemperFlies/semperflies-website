use std::{fs, path::PathBuf};

use tracing::warn;

#[tracing::instrument(name = "getting all images in directory path")]
pub fn all_images_in_directory(path: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut images = vec![];
    let image_exts = vec!["jpg", "jpeg", "png", "JPG"];
    let path = PathBuf::from(path);
    let entries = fs::read_dir(path).map_err(|err| {
        warn!("error with entries: {:?}", err);
        err
    })?;
    for file in entries {
        let path = file
            .map_err(|err| {
                warn!("error with entry: {:?}", err);
                err
            })?
            .path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if image_exts.contains(&ext.to_str().unwrap()) {
                    images.push(path)
                }
            }
        }
    }
    warn!("returning images: {:?}", images);
    Ok(images)
}

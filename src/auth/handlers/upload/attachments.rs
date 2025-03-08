use crate::{
    database::models::DBImageParams,
    routes::pages::{dedications::DEDICATIONS, patrol_log::logs::PATROL_LOG, support::SUPPORT},
    util,
};
use anyhow::anyhow;
use std::{
    fmt::Pointer,
    fs::{self, File},
    io::Write,
    ops::Deref,
    os::unix::fs::PermissionsExt,
};
use tracing::{error, warn};
use webp::WebPMemory;

use super::{multipart::UploadMultipartItemType, IMAGES_DIRECTORY};

#[derive(Debug)]
pub struct FileAttachment {
    name: String,
    bytes: Vec<u8>,
    pub new_name: Option<String>,
    pub alt: Option<String>,
    pub subtitle: Option<String>,
}

impl FileAttachment {
    fn attachments_path(subdir: Option<&str>, typ: &UploadMultipartItemType) -> String {
        format!(
            "./{}/{}{}",
            &IMAGES_DIRECTORY,
            match typ {
                UploadMultipartItemType::PatrolLog => PATROL_LOG,
                UploadMultipartItemType::Dedications => DEDICATIONS,
                UploadMultipartItemType::Support => SUPPORT,
            },
            match subdir {
                Some(dir) => format!("/{}", dir.replace(" ", "_")),
                None => "".to_string(),
            }
        )
    }

    pub fn new(name: &str, bytes: &[u8]) -> Self {
        Self {
            name: name.to_owned(),
            bytes: bytes.to_vec(),
            new_name: None,
            alt: None,
            subtitle: None,
        }
    }

    pub fn into_db_image_params(self, fs_path: &str) -> DBImageParams {
        DBImageParams {
            path: fs_path.to_string(),
            alt: self.alt.unwrap_or("".to_string()),
            subtitle: self.subtitle,
        }
    }

    #[tracing::instrument("remove from filesystem")]
    pub fn remove_from_filesys(
        subdir: Option<&str>,
        multipart_type: &UploadMultipartItemType,
    ) -> anyhow::Result<()> {
        let path_str = Self::attachments_path(subdir, multipart_type);

        let parent_path_str = path_str.rsplit_once('/').unwrap().0;

        let parent_metadata = fs::metadata(parent_path_str)?;
        let mut parent_perms = parent_metadata.permissions();
        parent_perms.set_readonly(false);

        let path = std::path::Path::new(&path_str);
        if !path.exists() {
            warn!("tried to delete path that doesn't exist: {path:?}");
            return Ok(());
        }

        if !path.is_dir() {
            return Err(anyhow!("path: {path:?} is not a directory"));
        }

        fs::remove_dir_all(path)?;

        Ok(())
    }

    #[tracing::instrument(name = "save attachment as webp image", skip(self))]
    pub fn save_as_webp(
        &self,
        subdir: Option<&str>,
        multipart_type: &UploadMultipartItemType,
    ) -> anyhow::Result<String> {
        let attachment_path_str = format!(
            "{}/{}",
            Self::attachments_path(subdir, multipart_type),
            self.new_name.to_owned().unwrap_or(self.name.to_owned())
        );
        let split = &attachment_path_str.rsplit_once('.').unwrap();
        let path_str = format!("{}.webp", split.0);
        let path = std::path::Path::new(&path_str);
        warn!("path: {path_str}");

        let webp_mem = util::bytes_to_webp(&self.bytes, split.1)?;
        match path.exists() {
            false => {
                warn!("file: {:?} does not exist, writing", path);
                let mut file = File::create_new(path).map_err(|err| anyhow!(err))?;
                file.write_all(&webp_mem.deref())
                    .map_err(|err| anyhow!(err))?;
            }
            true => {
                warn!("file: {:?} already exists, overwriting", path);
                fs::write(path, webp_mem.deref()).map_err(|err| {
                    let m = format!(
                        "there was a problem overriting the file: {:?}\n: {:?}",
                        path, err
                    );
                    error!(m);
                    anyhow!(m)
                })?;
            }
        }
        return Ok(path_str);
    }

    #[tracing::instrument(name = "save attachments to filesys", skip_all)]
    pub fn save_multiple_to_filesys(
        multiple: Vec<Self>,
        multipart_type: &UploadMultipartItemType,
        subdir: Option<&str>,
    ) -> anyhow::Result<Vec<DBImageParams>> {
        let mut return_params = vec![];
        let path_str = Self::attachments_path(subdir, multipart_type);

        let path = std::path::Path::new(&path_str);
        warn!("got path: {path:#?}");
        ensure_permissions_and_create_dirs(path, false)?;

        for attachment in multiple.into_iter() {
            let attachment_path_str = attachment
                .save_as_webp(subdir, multipart_type)
                .expect("failed to save image as webp");
            return_params.push(attachment.into_db_image_params(&attachment_path_str));
        }

        ensure_permissions_and_create_dirs(path, true)?;
        Ok(return_params)
    }
}

fn ensure_permissions_and_create_dirs(
    path: &std::path::Path,
    readonly: bool,
) -> anyhow::Result<()> {
    let mut current = path;

    while let Some(parent) = current.parent() {
        if parent.parent().is_none() {
            break;
        }
        match fs::metadata(parent) {
            Ok(metadata) => {
                if !metadata.permissions().readonly() {
                    warn!("Directory {:?} is writable", parent);
                } else {
                    metadata.permissions().set_readonly(readonly);
                    warn!("Fixed permissions for {:?}", parent);
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    fs::create_dir_all(parent)?;
                    warn!("Created parent directory: {:?}", parent);
                } else {
                    return Err(anyhow!("Error reading metadata for {:?}: {:?}", parent, e));
                }
            }
        }
        current = parent;
    }

    // Now ensure the target directory exists
    if !path.exists() {
        fs::create_dir(path)?;
        warn!("Created target directory: {:?}", path);
    }

    Ok(())
}

mod tests {
    use std::sync::LazyLock;

    use crate::{auth::handlers::upload::UploadMultipartItemType, TRACING};

    use super::FileAttachment;

    #[test]
    fn save_multiple_works() {
        LazyLock::force(&TRACING);
        let attachments = vec![
            FileAttachment {
                name: "test".to_string(),
                bytes: vec![0, 0, 0, 0],
                new_name: None,
                alt: None,
                subtitle: None,
            },
            FileAttachment {
                name: "test1".to_string(),
                bytes: vec![0, 0, 0, 0],
                new_name: None,
                alt: None,
                subtitle: None,
            },
            FileAttachment {
                name: "test2".to_string(),
                bytes: vec![0, 0, 0, 0],
                new_name: None,
                alt: None,
                subtitle: None,
            },
            FileAttachment {
                name: "test3".to_string(),
                bytes: vec![0, 0, 0, 0],
                new_name: None,
                alt: None,
                subtitle: None,
            },
        ];

        FileAttachment::save_multiple_to_filesys(
            attachments,
            &UploadMultipartItemType::Dedications,
            Some("unittest"),
        )
        .expect("failed");
    }
}

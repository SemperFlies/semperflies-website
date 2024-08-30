use crate::{
    database::models::DBImageParams,
    routes::pages::{dedications::DEDICATIONS, patrol_log::logs::PATROL_LOG},
};
use anyhow::anyhow;
use std::{
    fs::{self, File},
    io::Write,
};
use tracing::{error, warn};

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

    #[tracing::instrument(name = "save attachments to filesys", skip_all)]
    pub fn save_multiple_to_filesys(
        multiple: Vec<Self>,
        multipart_type: &UploadMultipartItemType,
        subdir: Option<&str>,
    ) -> anyhow::Result<Vec<DBImageParams>> {
        let mut return_params = vec![];

        let path_str = format!(
            "./{}/{}{}",
            &IMAGES_DIRECTORY,
            match multipart_type {
                UploadMultipartItemType::PatrolLog => PATROL_LOG,
                UploadMultipartItemType::Dedications => DEDICATIONS,
            },
            match subdir {
                Some(dir) => format!("/{}", dir),
                None => "".to_string(),
            }
        );
        warn!("got path str: {}", path_str);

        let parent_path_str = path_str.rsplit_once('/').unwrap().0;
        let public_path_str = parent_path_str.rsplit_once('/').unwrap().0;

        let public_metadata = fs::metadata(public_path_str)?;
        let mut public_perms = public_metadata.permissions();
        public_perms.set_readonly(false);

        let parent_metadata = fs::metadata(parent_path_str)?;
        let mut parent_perms = parent_metadata.permissions();
        parent_perms.set_readonly(false);

        let path = std::path::Path::new(&path_str);
        if !path.exists() {
            fs::create_dir(path).map_err(|err| {
                error!(
                    "there s an error when creating the posts assets directory: {:?}",
                    err
                );
                anyhow!(
                    "there was an error when creating the posts assets directory: {:?}",
                    err
                )
            })?;
        }

        for attachment in multiple.into_iter() {
            let attachment_path_str = format!(
                "{}/{}",
                path_str,
                attachment
                    .new_name
                    .to_owned()
                    .unwrap_or(attachment.name.to_owned())
            );
            let path = std::path::Path::new(&attachment_path_str);
            match path.exists() {
                false => {
                    warn!("file: {:?} does not exist, writing", path);
                    let mut file = File::create_new(path).map_err(|err| anyhow!(err))?;
                    file.write_all(&attachment.bytes)
                        .map_err(|err| anyhow!(err))?;
                }
                true => {
                    warn!("file: {:?} already exists, overwriting", path);
                    fs::write(path, &attachment.bytes).map_err(|err| {
                        error!(
                            "there was a problem overriting the file: {:?}\n: {:?}",
                            path, err
                        );
                        anyhow!(
                            "there was a problem overriting the file: {:?}\n: {:?}",
                            path,
                            err
                        )
                    })?;
                }
            }
            return_params.push(attachment.into_db_image_params(&attachment_path_str));
        }

        public_perms.set_readonly(true);
        parent_perms.set_readonly(true);
        Ok(return_params)
    }
}

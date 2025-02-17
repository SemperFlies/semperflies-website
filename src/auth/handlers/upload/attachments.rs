use crate::{
    database::models::DBImageParams,
    routes::pages::{dedications::DEDICATIONS, patrol_log::logs::PATROL_LOG, support::SUPPORT},
    util,
};
use anyhow::anyhow;
use std::{
    fs::{self, File},
    io::Write,
    ops::Deref,
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

        let mut all_parent_perms = vec![];
        let mut push_perms_to_all = |fnpath: &std::path::Path| {
            if !fnpath.exists() {
                fs::create_dir_all(fnpath)
                    .map_err(|err| {
                        error!(
                            "there s an error when creating the parent assets directory: {:?}",
                            err
                        );
                        anyhow!(
                            "there was an error when creating the parent assets directory: {:?}",
                            err
                        )
                    })
                    .expect("failed to create");
            }
            let metadata = fs::metadata(fnpath)
                .map_err(|e| {
                    warn!("probelm getting parent metadata: {e:?}");
                    e
                })
                .expect("failed to get metadata");
            let perms = metadata.permissions();
            all_parent_perms.push(perms);
        };

        push_perms_to_all(path);
        if let Some(parent) = path.parent() {
            warn!("got parent: {path:#?}");

            push_perms_to_all(parent);
        }

        warn!("changing permissions");
        all_parent_perms
            .iter_mut()
            .for_each(|p| p.set_readonly(false));

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
            let attachment_path_str = attachment
                .save_as_webp(subdir, multipart_type)
                .expect("failed to save image as webp");
            return_params.push(attachment.into_db_image_params(&attachment_path_str));
        }

        all_parent_perms
            .iter_mut()
            .for_each(|p| p.set_readonly(true));

        Ok(return_params)
    }
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

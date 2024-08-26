use crate::{
    auth::handlers::error::UploadError,
    database::{
        handles::DbData,
        models::{
            DBAddress, DBAddressParams, DBDedication, DBDedicationParams, DBPatrolLog,
            DBPatrolLogParams, DBResource, DBResourceParams, DBTestimonial, DBTestimonialParams,
        },
    },
    error::{DataApiReturn, InternalError},
    routes::pages::{
        debriefs::DEBRIEFS,
        dedications::DEDICATIONS,
        patrol_log::logs::PATROL_LOG,
        support::{Address, SUPPORT},
    },
    state::SharedState,
};
use anyhow::anyhow;
use axum::{
    extract::{Multipart, Path, State},
    http::Response,
    response::IntoResponse,
    Form,
};
use chrono::NaiveDate;
use reqwest::StatusCode;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::Write,
    ops::Deref,
};
use tracing::{debug, error, warn};

#[derive(Debug)]
pub(super) enum UploadFormItemType {
    Support,
    Debriefs,
}

#[derive(Debug)]
pub(super) enum UploadMultipartItemType {
    PatrolLog,
    Dedications,
}

#[derive(Debug)]
pub enum UploadItem {
    Address(DBAddressParams),
    Support(DBResourceParams),
    Debrief(DBTestimonialParams),
    PatrolLog(DBPatrolLogParams),
    Dedication(DBDedicationParams),
}

#[derive(Debug)]
struct FileAttachment {
    name: String,
    bytes: Vec<u8>,
}

const IMAGES_DIRECTORY: &str = "public/assets/images";

impl FileAttachment {
    #[tracing::instrument(name = "save attachments to filesys", skip_all)]
    fn save_multiple_to_filesys(
        mut multiple: Vec<Self>,
        multipart_type: &UploadMultipartItemType,
        subdir: Option<&str>,
    ) -> anyhow::Result<Vec<String>> {
        let mut return_urls = vec![];

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

        for attachment in multiple.iter_mut() {
            let attachment_path_str = format!("{}/{}", path_str, attachment.name);
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
            return_urls.push(attachment_path_str);
        }

        public_perms.set_readonly(true);
        parent_perms.set_readonly(true);
        Ok(return_urls)
    }
}

pub trait UploadItemType<T>
where
    T: std::fmt::Debug,
{
    fn try_from_str(str: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
    async fn into_item(self, form_or_multipart: T) -> anyhow::Result<UploadItem>;
}

impl UploadItemType<Value> for UploadFormItemType {
    fn try_from_str(str: &str) -> anyhow::Result<Self> {
        warn!("getting item: {}", str);
        match str {
            _ if str == SUPPORT => Ok(Self::Support),
            _ if str == DEBRIEFS => Ok(Self::Debriefs),
            other => {
                warn!("none found for: {}", other);
                Err(anyhow!("{} is not a valid upload form item", other))
            }
        }
    }
    async fn into_item(self, form: Value) -> anyhow::Result<UploadItem> {
        match self {
            Self::Support => {
                let physical_address = match form.get("city") {
                    Some(city) => {
                        let state = form
                            .get("state")
                            .ok_or(UploadError::user_facing("state is none"))?
                            .to_string();
                        let zip = form
                            .get("zip")
                            .ok_or(UploadError::user_facing("zip is none"))?
                            .to_string();
                        let line_1 = form
                            .get("line1")
                            .ok_or(UploadError::user_facing("line1 is none"))?
                            .to_string();
                        let line_2 = form.get("line2").and_then(|v| Some(v.to_string()));
                        Some(Address {
                            city: city.to_string(),
                            state,
                            zip,
                            line_1,
                            line_2,
                        })
                    }
                    None => None,
                };

                let missions: Vec<String> = match form.get("missions") {
                    Some(mis) => serde_json::from_value::<Vec<String>>(mis.to_owned())?,
                    None => vec![],
                };

                let res = DBResourceParams {
                    name: form
                        .get("name")
                        .ok_or(UploadError::user_facing("name is none"))?
                        .to_string(),
                    description: form
                        .get("description")
                        .ok_or(UploadError::user_facing("description is none"))?
                        .to_string(),
                    missions,
                    phone: form.get("phone").and_then(|v| Some(v.to_string())),
                    email: form.get("email").and_then(|v| Some(v.to_string())),
                    address: physical_address.and_then(|add| {
                        Some(DBAddressParams {
                            city: add.city,
                            state: add.state,
                            zip: add.zip,
                            line_1: add.line_1,
                            line_2: add.line_2,
                        })
                    }),
                };
                Ok(UploadItem::Support(res))
            }

            Self::Debriefs => {
                let test = DBTestimonialParams {
                    firstname: form
                        .get("firstname")
                        .ok_or(UploadError::user_facing("firstname is none"))?
                        .to_string(),
                    lastname: form
                        .get("lastname")
                        .ok_or(UploadError::user_facing("lastname is none"))?
                        .to_string(),
                    content: form
                        .get("content")
                        .ok_or(UploadError::user_facing("content is none"))?
                        .to_string(),
                    bio: form.get("bio").and_then(|v| Some(v.to_string())),
                };
                Ok(UploadItem::Debrief(test))
            }
        }
    }
}

impl UploadItemType<Multipart> for UploadMultipartItemType {
    fn try_from_str(str: &str) -> anyhow::Result<Self> {
        warn!("getting item: {}", str);
        match str {
            _ if str == PATROL_LOG => Ok(Self::PatrolLog),
            _ if str == DEDICATIONS => Ok(Self::Dedications),
            other => {
                warn!("none found for: {}", other);
                Err(anyhow!("{} is not a valid upload multipart item", other))
            }
        }
    }
    async fn into_item(self, mut multipart: Multipart) -> anyhow::Result<UploadItem> {
        match self {
            Self::PatrolLog => {
                let mut date = Option::<NaiveDate>::None;
                let mut heading = Option::<String>::None;
                let mut description = Option::<String>::None;
                let mut attachments = vec![];
                while let Some(field) = multipart.next_field().await? {
                    let fieldname = field
                        .name()
                        .ok_or(anyhow!("no name on field: {:?}", field))?;

                    match fieldname {
                        "date" => {
                            let str = field.text().await?;
                            date = Some(naive_date_from_str(str.as_str())?);
                        }
                        "heading" => {
                            heading = Some(field.text().await?);
                        }
                        "description" => {
                            description = Some(field.text().await?);
                        }
                        "images" => {
                            let attachment = FileAttachment {
                                name: field
                                    .file_name()
                                    .ok_or(anyhow!("no file name on image"))?
                                    .to_string(),
                                bytes: field.bytes().await?.deref().to_vec(),
                            };
                            attachments.push(attachment);
                        }
                        other => {
                            warn!("{} is not a supported fieldname", other);
                        }
                    }
                }

                let heading = heading.expect("no heading");
                let img_urls =
                    FileAttachment::save_multiple_to_filesys(attachments, &self, Some(&heading))?;

                let log = DBPatrolLogParams {
                    heading,
                    description: description.expect("no description"),
                    date: date.expect("no date"),
                    img_urls,
                };

                Ok(UploadItem::PatrolLog(log))
            }

            Self::Dedications => {
                let mut birth = Option::<NaiveDate>::None;
                let mut death = Option::<NaiveDate>::None;
                let mut name = Option::<String>::None;
                let mut bio = Option::<String>::None;
                let mut attachments = vec![];
                while let Some(field) = multipart.next_field().await? {
                    let fieldname = field
                        .name()
                        .ok_or(anyhow!("no name on field: {:?}", field))?;
                    warn!("processing field: {:?}", fieldname);

                    match fieldname {
                        "birth" => {
                            let str = field.text().await?;
                            birth = Some(naive_date_from_str(str.as_str())?);
                        }
                        "death" => {
                            let str = field.text().await?;
                            death = Some(naive_date_from_str(str.as_str())?);
                        }
                        "name" => {
                            name = Some(field.text().await?);
                        }
                        "bio" => {
                            bio = Some(field.text().await?);
                        }
                        "images" => {
                            let attach_name = field
                                .file_name()
                                .ok_or(anyhow!("no file name on image"))?
                                .to_string();
                            warn!("got attachment name: {:?}", attach_name);
                            let bytes = field
                                .bytes()
                                .await
                                .map_err(|err| {
                                    anyhow!(
                                        "error occurred when getting attachment bytes: {:?}",
                                        err
                                    )
                                })?
                                .deref()
                                .to_vec();
                            warn!("got attachment bytes len: {}", bytes.len());
                            let attachment = FileAttachment {
                                name: attach_name,
                                bytes,
                            };
                            attachments.push(attachment);
                        }
                        other => {
                            warn!("{} is not a supported fieldname", other);
                        }
                    }
                }
                warn!("outside of field processing loop");
                let name = name.expect("no name");
                let img_urls =
                    FileAttachment::save_multiple_to_filesys(attachments, &self, Some(&name))?;

                let ded = DBDedicationParams {
                    name,
                    bio: bio.expect("no bio"),
                    birth: birth.expect("no birth"),
                    death: death.expect("no death"),
                    img_urls,
                };
                Ok(UploadItem::Dedication(ded))
            }
        }
    }
}

#[tracing::instrument(name = "upload handler", skip(data))]
pub async fn upload_form_handler(
    Path(item_str): Path<String>,
    State(data): State<SharedState>,
    Form(form): Form<Value>,
) -> Result<impl IntoResponse, DataApiReturn> {
    let item =
        UploadFormItemType::try_from_str(item_str.as_str()).expect("failed to get upload item");
    let success_message = format!("succesfully uploaded {}", item_str);
    warn!("got item: {:?}", item);
    match item.into_item(form).await {
        Ok(uploadable) => {
            let r = data.read().await;
            warn!("inserting: {:?}", uploadable);

            match uploadable {
                UploadItem::Address(add) => {
                    DBAddress::insert_one(add, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }
                UploadItem::Support(res) => {
                    DBResource::insert_one(res, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }

                UploadItem::Debrief(test) => {
                    DBTestimonial::insert_one(test, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }
                UploadItem::Dedication(ded) => {
                    DBDedication::insert_one(ded, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }
                UploadItem::PatrolLog(log) => {
                    DBPatrolLog::insert_one(log, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }
            }

            let response = Response::new(success_message);
            Ok(response)
        }
        Err(err) => {
            warn!("returning err: {:?}", err);
            let r = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(err.to_string())
                .unwrap();
            return Ok(r);
        }
    }
}

pub async fn upload_multipart_handler(
    State(data): State<SharedState>,
    Path(item_str): Path<String>,
    multipart: Multipart,
) -> Result<impl IntoResponse, DataApiReturn> {
    let item = UploadMultipartItemType::try_from_str(item_str.as_str())
        .expect("failed to get upload item");
    let success_message = format!("succesfully uploaded {}", item_str);
    warn!("got item: {:?}", item);
    match item.into_item(multipart).await {
        Ok(uploadable) => {
            let r = data.read().await;
            warn!("inserting: {:?}", uploadable);

            match uploadable {
                UploadItem::Address(add) => {
                    DBAddress::insert_one(add, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }
                UploadItem::Support(res) => {
                    DBResource::insert_one(res, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }

                UploadItem::Debrief(test) => {
                    DBTestimonial::insert_one(test, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }
                UploadItem::Dedication(ded) => {
                    DBDedication::insert_one(ded, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }
                UploadItem::PatrolLog(log) => {
                    DBPatrolLog::insert_one(log, &r.db)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                }
            }

            let response = Response::new(success_message);
            Ok(response)
        }
        Err(err) => {
            warn!("returning err: {:?}", err);
            let r = Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(err.to_string())
                .unwrap();
            return Ok(r);
        }
    }
}

fn naive_date_from_str(str: &str) -> anyhow::Result<NaiveDate> {
    let onllynums: String = str
        .to_string()
        .chars()
        .into_iter()
        .filter(|c| c.is_numeric())
        .collect();
    let year: i32 = onllynums.chars().take(4).collect::<String>().parse()?;
    let month: u32 = onllynums
        .chars()
        .skip(4)
        .take(2)
        .collect::<String>()
        .parse()?;
    let day: u32 = onllynums
        .chars()
        .skip(6)
        .take(2)
        .collect::<String>()
        .parse()?;
    NaiveDate::from_ymd_opt(year, month, day).ok_or(anyhow!(
        "could not build naive date from {:?}",
        (year, month, day)
    ))
}

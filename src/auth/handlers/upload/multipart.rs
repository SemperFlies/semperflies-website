use crate::{
    auth::handlers::{
        error::UploadError,
        upload::{attachments::FileAttachment, naive_date_from_str},
    },
    database::models::{
        DBDedication, DBDedicationParams, DBImage, DBImageParams, DBPatrolLog, DBPatrolLogParams,
    },
    error::{DataApiReturn, InternalError},
    routes::pages::{dedications::DEDICATIONS, patrol_log::logs::PATROL_LOG},
    state::SharedState,
};
use anyhow::anyhow;
use axum::{
    extract::{multipart::Field, Multipart, Path, State},
    http::Response,
    response::IntoResponse,
};
use chrono::NaiveDate;
use reqwest::StatusCode;
use std::ops::Deref;
use tracing::warn;

use super::{UploadItem, UploadItemType, IMAGES_DIRECTORY};

#[derive(Debug)]
pub enum UploadMultipartItemType {
    PatrolLog,
    Dedications,
}

// impl UploadMultipartItemType {
//     pub fn images_path(&self, item: &UploadItem) -> String {
//         let subdir: &String = match &item {
//             UploadItem::PatrolLog(item) => &item.heading,
//             UploadItem::Dedication(item) => &item.name,
//             other => panic!("{other:?} should not have been passed to this method"),
//         };
//
//         format!(
//             "./{}/{}{}",
//             &IMAGES_DIRECTORY,
//             match self {
//                 UploadMultipartItemType::PatrolLog => PATROL_LOG,
//                 UploadMultipartItemType::Dedications => DEDICATIONS,
//             },
//             subdir
//         )
//     }
// }

async fn handle_other(other: &str, field: Field<'_>, attachments: &mut Vec<FileAttachment>) {
    match other.split_once('_') {
        None => {
            warn!("{} is not a supported fieldname", other);
        }
        Some((attribute, idx)) => {
            let idx: usize = idx.parse().expect("could not parse idx into usize");
            let text = match field.text().await {
                Ok(t) => {
                    if t.is_empty() {
                        None
                    } else {
                        Some(t)
                    }
                }
                Err(err) => {
                    warn!("there was a problem getting text: {:?}", err);
                    None
                }
            };
            if attachments.len() - 1 < idx {
                warn!("attachments and info not long enough")
            } else {
                match attribute {
                    "name" => {
                        attachments[idx].new_name = text;
                    }
                    "alt" => {
                        attachments[idx].alt = text;
                    }
                    "subtitle" => {
                        attachments[idx].subtitle = text;
                    }
                    o => warn!("unexpected attribute: {}", o),
                }
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
        warn!("coercing multipart: {:?}", multipart);
        match self {
            Self::PatrolLog => {
                let mut date = Option::<NaiveDate>::None;
                let mut heading = Option::<String>::None;
                let mut description = Option::<String>::None;
                let mut attachments = vec![];
                while let Some(field) = multipart.next_field().await? {
                    let fieldname = field
                        .name()
                        .ok_or(anyhow!("no name on field: {:?}", field))?
                        .to_owned();

                    match fieldname.as_str() {
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
                            let attachment = FileAttachment::new(
                                field
                                    .file_name()
                                    .ok_or(anyhow!("no file name on image"))?
                                    .to_owned()
                                    .as_str(),
                                field.bytes().await?.deref(),
                            );

                            attachments.push(attachment);
                        }

                        other => handle_other(other, field, &mut attachments).await,
                    }
                }

                let heading = heading.expect("no heading");
                let mut img_params = vec![];
                if !attachments.is_empty() {
                    img_params = FileAttachment::save_multiple_to_filesys(
                        attachments,
                        &self,
                        Some(&heading),
                    )
                    .expect("failed to save attachments to filesys");
                }

                let log = DBPatrolLogParams {
                    heading,
                    description: description.expect("no description"),
                    date: date.expect("no date"),
                    img_params,
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
                        .ok_or(anyhow!("no name on field: {:?}", field))?
                        .to_owned();
                    warn!("processing field: {:?}", fieldname);

                    match fieldname.as_str() {
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
                            let attachment = FileAttachment::new(
                                field
                                    .file_name()
                                    .ok_or(anyhow!("no file name on image"))?
                                    .to_owned()
                                    .as_str(),
                                field.bytes().await?.deref(),
                            );
                            attachments.push(attachment);
                        }
                        other => handle_other(other, field, &mut attachments).await,
                    }
                }
                warn!("outside of field processing loop");
                let name = name.expect("no name");
                let mut img_params = vec![];
                if !attachments.is_empty() {
                    img_params =
                        FileAttachment::save_multiple_to_filesys(attachments, &self, Some(&name))
                            .expect("failed to save attachments to filesys");
                }

                let ded = DBDedicationParams {
                    name,
                    bio: bio.expect("no bio"),
                    birth: birth.expect("no birth"),
                    death: death.expect("no death"),
                    img_params,
                };
                Ok(UploadItem::Dedication(ded))
            }
        }
    }
}

#[tracing::instrument(name = "upload multipart handler", skip(data))]
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
                UploadItem::Dedication(ded) => {
                    DBImage::insert_multiple_with_images::<DBDedication, DBDedicationParams>(
                        &r.db,
                        vec![ded],
                    )
                    .await
                    .map_err(|err| {
                        warn!("error: {:?}", err);
                        UploadError::from(err).into_data_api_return()
                    })?;
                }

                UploadItem::PatrolLog(log) => {
                    DBImage::insert_multiple_with_images::<DBPatrolLog, DBPatrolLogParams>(
                        &r.db,
                        vec![log],
                    )
                    .await
                    .map_err(|err| {
                        warn!("error: {:?}", err);
                        UploadError::from(err).into_data_api_return()
                    })?;
                }
                other => {
                    let m = format!(
                        "{:?} is not a supported upload type for a multipart upload",
                        other
                    );
                    warn!(m);
                    return Ok(Response::new(m));
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

use crate::{
    auth::handlers::{
        error::UploadError,
        upload::{attachments::FileAttachment, naive_date_from_str},
    },
    database::{
        handles::DbData,
        models::{
            DBAddressParams, DBDedication, DBDedicationParams, DBImage, DBImageParams, DBPatrolLog,
            DBPatrolLogParams, DBResource, DBResourceParams,
        },
    },
    error::{DataApiReturn, InternalError},
    routes::pages::{dedications::DEDICATIONS, patrol_log::logs::PATROL_LOG, support::SUPPORT},
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
    Support,
}

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

fn none_if_empty_string(str: String) -> Option<String> {
    if str.trim().is_empty() {
        None
    } else {
        Some(str)
    }
}

impl UploadItemType<Multipart> for UploadMultipartItemType {
    fn try_from_str(str: &str) -> anyhow::Result<Self> {
        warn!("getting item: {}", str);
        match str {
            _ if str == PATROL_LOG => Ok(Self::PatrolLog),
            _ if str == DEDICATIONS => Ok(Self::Dedications),
            _ if str == SUPPORT => Ok(Self::Support),
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

                let description = description
                    .expect("expected description")
                    .replace("\n", "<br/>");

                let log = DBPatrolLogParams {
                    heading,
                    description,
                    date: date.expect("no date"),
                    img_params,
                };

                Ok(UploadItem::PatrolLog(log))
            }

            Self::Dedications => {
                let mut birth = Option::<NaiveDate>::None;
                let mut death = Option::<NaiveDate>::None;
                let mut names = Option::<Vec<String>>::None;
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
                        "names[]" => {
                            let names_str = field.text().await?;
                            warn!("got names str: {names_str}");
                            names = Some(names_str.split(',').map(|s| s.to_string()).collect())
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
                let names = names.expect("no names");
                let mut img_params = vec![];
                if !attachments.is_empty() {
                    img_params = FileAttachment::save_multiple_to_filesys(
                        attachments,
                        &self,
                        Some(&names.join("-")),
                    )
                    .expect("failed to save attachments to filesys");
                }

                let bio = bio.expect("expected bio").replace("\n", "<br/>");
                let ded = DBDedicationParams {
                    names,
                    bio,
                    birth: birth.expect("no birth"),
                    death: death.expect("no death"),
                    img_params,
                };
                Ok(UploadItem::Dedication(ded))
            }
            Self::Support => {
                let mut name = Option::<String>::None;
                let mut description = Option::<String>::None;

                let mut city = Option::<String>::None;
                let mut state = Option::<String>::None;
                let mut zip = Option::<String>::None;
                let mut line_1 = Option::<String>::None;
                let mut line_2 = Option::<String>::None;

                let mut website_url = Option::<String>::None;
                let mut phone = Option::<String>::None;
                let mut email = Option::<String>::None;

                let mut twitter = Option::<String>::None;
                let mut facebook = Option::<String>::None;
                let mut youtube = Option::<String>::None;
                let mut linkedin = Option::<String>::None;
                let mut threads = Option::<String>::None;
                let mut instagram = Option::<String>::None;

                let mut missions = vec![];
                let mut attachments = vec![];
                while let Some(field) = multipart.next_field().await? {
                    let fieldname = field
                        .name()
                        .ok_or(anyhow!("no name on field: {:?}", field))?
                        .to_owned();
                    warn!("processing field: {:?}", fieldname);

                    match fieldname.as_str() {
                        "name" => {
                            name = Some(field.text().await?);
                        }
                        "description" => {
                            let description_str = field.text().await?;
                            description = Some(description_str.replace("\n", "<br/>"));
                        }

                        "city" => {
                            city = none_if_empty_string(field.text().await?);
                        }
                        "state" => {
                            state = none_if_empty_string(field.text().await?);
                        }
                        "zip" => {
                            zip = none_if_empty_string(field.text().await?);
                        }
                        "line1" => {
                            line_1 = none_if_empty_string(field.text().await?);
                        }
                        "line2" => {
                            line_2 = none_if_empty_string(field.text().await?);
                        }

                        "website" => {
                            website_url = none_if_empty_string(field.text().await?);
                        }

                        "phone" => {
                            phone = none_if_empty_string(field.text().await?);

                            if let Some(ref ph) = phone {
                                match ph.len() {
                                    10 => {
                                        phone = Some(format!(
                                            "({})-{}-{}",
                                            &ph[..3],
                                            &ph[3..6],
                                            &ph[6..10],
                                        ))
                                    }
                                    11 => {
                                        phone = Some(format!(
                                            "{}-({})-{}-{}",
                                            &ph[..=0],
                                            &ph[1..4],
                                            &ph[4..7],
                                            &ph[7..11],
                                        ))
                                    }
                                    other => warn!("{} is not a valid phone number len", other),
                                }
                            }
                        }
                        "email" => {
                            email = none_if_empty_string(field.text().await?);
                        }

                        "twitter" => {
                            twitter = none_if_empty_string(field.text().await?);
                        }
                        "facebook" => {
                            facebook = none_if_empty_string(field.text().await?);
                        }
                        "youtube" => {
                            youtube = none_if_empty_string(field.text().await?);
                        }
                        "linkedin" => {
                            linkedin = none_if_empty_string(field.text().await?);
                        }
                        "threads" => {
                            threads = none_if_empty_string(field.text().await?);
                        }
                        "instagram" => {
                            instagram = none_if_empty_string(field.text().await?);
                        }

                        "missions[]" => {
                            let missions_str = field.text().await?;
                            warn!("got missions str: {missions_str}");
                            missions = missions_str
                                .split(',')
                                .filter_map(|s| {
                                    if s.trim().is_empty() {
                                        None
                                    } else {
                                        Some(s.to_string())
                                    }
                                })
                                .collect();
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

                let address =
                    if let (Some(c), Some(s), Some(z), Some(l1)) = (city, state, zip, line_1) {
                        Some(DBAddressParams {
                            city: c,
                            state: s,
                            zip: z,
                            line_1: l1,
                            line_2,
                        })
                    } else {
                        None
                    };

                let mut img_params = vec![];
                if !attachments.is_empty() {
                    img_params =
                        FileAttachment::save_multiple_to_filesys(attachments, &self, Some(&name))
                            .expect("failed to save attachments to filesys");
                }

                let res = DBResourceParams {
                    name,
                    img_params,
                    description: description.expect("no description"),
                    missions,
                    website_url,
                    phone,
                    email,
                    address,
                    instagram,
                    facebook,
                    youtube,
                    linkedin,
                    threads,
                    twitter,
                };
                Ok(UploadItem::Support(res))
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

                UploadItem::Support(support) => {
                    DBResource::insert_one(support, &r.db)
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

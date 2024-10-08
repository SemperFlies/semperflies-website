use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
};
use tracing::warn;
use uuid::Uuid;

use crate::{
    auth::handlers::upload::attachments::FileAttachment,
    database::{
        handles::DbData,
        models::{DBDedication, DBImage, DBPatrolLog, DBResource, DBTestimonial},
    },
    error::{DataApiReturn, InternalError},
    state::SharedState,
};

use super::{
    error::UploadError,
    upload::{UploadFormItemType, UploadItemType, UploadMultipartItemType},
};

#[derive(Debug)]
enum GeneralItem {
    Form(UploadFormItemType),
    Multi(UploadMultipartItemType),
}

impl TryFrom<&str> for GeneralItem {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if let Some(item) = UploadFormItemType::try_from_str(value).ok() {
            return Ok(Self::Form(item));
        } else if let Some(item) = UploadMultipartItemType::try_from_str(value).ok() {
            return Ok(Self::Multi(item));
        }

        Err(anyhow!("{} is not a supported item", value))
    }
}

#[tracing::instrument(name = "deletion handler", skip(data))]
pub async fn delete_item_handler(
    Path((item_str, id)): Path<(String, Uuid)>,
    State(data): State<SharedState>,
) -> Result<impl IntoResponse, DataApiReturn> {
    let item = GeneralItem::try_from(item_str.as_str()).expect("failed to get item");
    let success_message = format!(
        "succesfully deleted {}",
        item_str[..item_str.len() - 1].to_string()
    );
    warn!("got item: {:?}", item);

    let r = data.read().await;
    let pool = &r.db;
    match item {
        GeneralItem::Form(i) => match i {
            UploadFormItemType::Debriefs => {
                DBTestimonial::delete_one_with_id(id, pool)
                    .await
                    .map_err(|err| UploadError::from(err).into_data_api_return())?;
            }
        },
        GeneralItem::Multi(i) => {
            let (img_ids, subdir) = match i {
                UploadMultipartItemType::Support => {
                    let ret = DBResource::delete_one_with_id(id, pool)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                    (ret.img_ids, ret.name)
                }
                UploadMultipartItemType::PatrolLog => {
                    let ret = DBPatrolLog::delete_one_with_id(id, pool)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                    (ret.img_ids, ret.heading)
                }
                UploadMultipartItemType::Dedications => {
                    let ret = DBDedication::delete_one_with_id(id, pool)
                        .await
                        .map_err(|err| UploadError::from(err).into_data_api_return())?;
                    (ret.img_ids, ret.names.join("-"))
                }
            };
            DBImage::delete_many_by_ids(img_ids, pool)
                .await
                .map_err(|err| UploadError::from(err).into_data_api_return())?;
            FileAttachment::remove_from_filesys(Some(&subdir), &i)
                .map_err(|err| UploadError::from(err).into_data_api_return())?;
        }
    }

    let response = Response::new(success_message);
    Ok(response)
}

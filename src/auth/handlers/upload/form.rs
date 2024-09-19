use crate::{
    auth::handlers::error::UploadError,
    database::{
        handles::DbData,
        models::{
            DBAddress, DBAddressParams, DBResource, DBResourceParams, DBTestimonial,
            DBTestimonialParams,
        },
    },
    error::{DataApiReturn, DataResponse, InternalError},
    routes::pages::{
        debriefs::DEBRIEFS,
        support::{Address, SUPPORT},
    },
    state::SharedState,
};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    http::Response,
    response::IntoResponse,
    Form,
};
use reqwest::StatusCode;
use serde_json::Value;
use tracing::warn;

use super::{UploadItem, UploadItemType};

#[derive(Debug)]
pub enum UploadFormItemType {
    // Support,
    Debriefs,
}

#[tracing::instrument(name = "upload form handler", skip(data))]
pub async fn upload_form_handler(
    Path(item_str): Path<String>,
    State(data): State<SharedState>,
    Form(form): Form<Value>,
) -> DataApiReturn {
    let item =
        UploadFormItemType::try_from_str(item_str.as_str()).expect("failed to get upload item");
    let success_message = format!("succesfully uploaded {}", item_str);
    warn!("got item: {:?}", item);
    match item.into_item(form).await {
        Ok(uploadable) => {
            let r = data.read().await;
            warn!("inserting: {:?}", uploadable);

            if let Err(e) = match uploadable {
                UploadItem::Address(add) => match DBAddress::insert_one(add, &r.db).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(UploadError::from(e)),
                },
                UploadItem::Support(res) => match DBResource::insert_one(res, &r.db).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(UploadError::from(e)),
                },
                UploadItem::Debrief(test) => match DBTestimonial::insert_one(test, &r.db).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(UploadError::from(e)),
                },
                other => {
                    let msg = format!(
                        "{:?} is not a supported upload type for a form upload",
                        other
                    );
                    warn!("{}", msg);
                    return DataResponse::success(msg.as_str());
                }
            } {
                return DataResponse::error(format!("Failed to upload item: {e:?}"), None);
            }
            DataResponse::success(&success_message)
        }
        Err(err) => {
            warn!("returning err: {:?}", err);
            return DataResponse::error(err, None);
        }
    }
}

fn get_optional_string_from_form(fieldname: &str, form: &Value) -> Option<String> {
    form.get(fieldname).and_then(|v| {
        let str: String = serde_json::from_value(v.to_owned())
            .map_err(|err| {
                warn!("problem with value: {:?}", err);
            })
            .ok()?;
        if !str.trim().is_empty() {
            Some(str)
        } else {
            None
        }
    })
}

impl UploadItemType<Value> for UploadFormItemType {
    fn try_from_str(str: &str) -> anyhow::Result<Self> {
        warn!("getting item: {}", str);
        match str {
            // _ if str == SUPPORT => Ok(Self::Support),
            _ if str == DEBRIEFS => Ok(Self::Debriefs),
            other => {
                warn!("none found for: {}", other);
                Err(anyhow!("{} is not a valid upload form item", other))
            }
        }
    }
    async fn into_item(self, form: Value) -> anyhow::Result<UploadItem> {
        match self {
            Self::Debriefs => {
                let firstname = get_optional_string_from_form("firstname", &form)
                    .ok_or(UploadError::user_facing("firstname is none"))?;
                let lastname = get_optional_string_from_form("lastname", &form)
                    .ok_or(UploadError::user_facing("lastname is none"))?;
                let mut content = get_optional_string_from_form("content", &form)
                    .ok_or(UploadError::user_facing("content is none"))?;
                content = content.replace("\n", "<br/>");
                let bio = get_optional_string_from_form("bio", &form);
                let test = DBTestimonialParams {
                    firstname,
                    lastname,
                    content,
                    bio,
                };
                Ok(UploadItem::Debrief(test))
            }
        }
    }
}

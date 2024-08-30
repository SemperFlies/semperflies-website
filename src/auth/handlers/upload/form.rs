use crate::{
    auth::handlers::error::UploadError,
    database::{
        handles::DbData,
        models::{
            DBAddress, DBAddressParams, DBResource, DBResourceParams, DBTestimonial,
            DBTestimonialParams,
        },
    },
    error::{DataApiReturn, InternalError},
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
    Support,
    Debriefs,
}

#[tracing::instrument(name = "upload form handler", skip(data))]
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
                other => {
                    let m = format!(
                        "{:?} is not a supported upload type for a form upload",
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

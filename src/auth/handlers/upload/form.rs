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
                let physical_address = match get_optional_string_from_form("city", &form) {
                    Some(city) => {
                        let state = get_optional_string_from_form("state", &form)
                            .ok_or(UploadError::user_facing("state is none"))?;
                        let zip = get_optional_string_from_form("zip", &form)
                            .ok_or(UploadError::user_facing("zip is none"))?;
                        let line_1 = get_optional_string_from_form("line1", &form)
                            .ok_or(UploadError::user_facing("line_1 is none"))?;
                        let line_2 = get_optional_string_from_form("line2", &form);
                        Some(Address {
                            city,
                            state,
                            zip,
                            line_1,
                            line_2,
                        })
                    }
                    None => None,
                };

                let missions: Vec<String> = match form.get("missions[]") {
                    Some(mis) => {
                        warn!("got serialized missions: {:?}", mis);
                        let mis = serde_json::from_value::<String>(mis.to_owned())?;
                        mis.split(',')
                            .filter_map(|m| {
                                if !m.trim().is_empty() {
                                    Some(m.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                    None => {
                        warn!("got no missions");
                        vec![]
                    }
                };
                warn!("got deserialized missions: {:?}", missions);

                let website_url = get_optional_string_from_form("website", &form);
                let mut phone = get_optional_string_from_form("phone", &form);

                if let Some(ref ph) = phone {
                    match ph.len() {
                        10 => phone = Some(format!("({})-{}-{}", &ph[..3], &ph[3..6], &ph[6..10],)),
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
                warn!("phone: {:?}", phone);

                let email = get_optional_string_from_form("email", &form);
                let description = get_optional_string_from_form("description", &form)
                    .ok_or(UploadError::user_facing("description is none"))?;
                let name = get_optional_string_from_form("name", &form)
                    .ok_or(UploadError::user_facing("name is none"))?;

                let address = physical_address.and_then(|add| {
                    Some(DBAddressParams {
                        city: add.city,
                        state: add.state,
                        zip: add.zip,
                        line_1: add.line_1,
                        line_2: add.line_2,
                    })
                });

                let res = DBResourceParams {
                    name,
                    description,
                    missions,
                    website_url,
                    phone,
                    email,
                    address,
                };
                Ok(UploadItem::Support(res))
            }

            Self::Debriefs => {
                let firstname = get_optional_string_from_form("firstname", &form)
                    .ok_or(UploadError::user_facing("firstname is none"))?;
                let lastname = get_optional_string_from_form("lastname", &form)
                    .ok_or(UploadError::user_facing("lastname is none"))?;
                let content = get_optional_string_from_form("content", &form)
                    .ok_or(UploadError::user_facing("content is none"))?;
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

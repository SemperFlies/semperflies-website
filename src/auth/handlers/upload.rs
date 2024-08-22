use super::error::UploadResult;
use crate::{
    auth::handlers::error::UploadError,
    error::{DataApiReturn, InternalError},
    routes::pages::uploadables::*,
    state::SharedState,
};
use anyhow::anyhow;
use axum::{
    extract::{Path, State},
    http::Response,
    response::IntoResponse,
    Form,
};
use chrono::NaiveDate;
use reqwest::StatusCode;
use serde_json::{json, Value};
use std::str::FromStr;
use tracing::{debug, warn};

#[derive(Debug)]
enum UploadItem {
    Support,
    Debriefs,
    PatrolLogs,
    FallenBrothers,
}

impl TryFrom<&str> for UploadItem {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        warn!("getting item: {}", value);
        match value {
            "support" => Ok(Self::Support),
            "debriefs" => Ok(Self::Debriefs),
            "patrol_logs" => Ok(Self::PatrolLogs),
            "fallen_brothers" => Ok(Self::FallenBrothers),
            other => {
                warn!("none found for: {}", other);
                Err(anyhow!("{} is not a valid upload item", other))
            }
        }
    }
}

trait Uploadable: std::fmt::Debug {}
impl Uploadable for Testimonial {}
impl Uploadable for Dedication {}
impl Uploadable for Log {}
impl Uploadable for SupportResource {}

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

impl UploadItem {
    fn form_into_(&self, form: Value) -> UploadResult<Box<dyn Uploadable>> {
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

                let res = SupportResource {
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
                    physical_address,
                };
                Ok(Box::new(res))
            }
            //
            Self::Debriefs => {
                let test = Testimonial {
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
                Ok(Box::new(test))
            }

            Self::PatrolLogs => {
                let date_str = form
                    .get("date")
                    .ok_or(UploadError::user_facing("date is none"))?;
                let date = naive_date_from_str(date_str.to_string().as_str())?;
                // NEED TO ADD IMAGE UPLOADING
                let carousel = crate::components::carousel::CarouselTemplate { images: vec![] };
                let log = Log {
                    heading: form
                        .get("heading")
                        .ok_or(UploadError::user_facing("heading is none"))?
                        .to_string(),
                    description: form
                        .get("description")
                        .ok_or(UploadError::user_facing("description is none"))?
                        .to_string(),
                    date,
                    carousel,
                };
                Ok(Box::new(log))
            }

            Self::FallenBrothers => {
                let bdate_str = form
                    .get("birth")
                    .ok_or(UploadError::user_facing("birth is none"))?
                    .to_string();
                warn!("dbay_str: {:?}", bdate_str);
                let birth = naive_date_from_str(bdate_str.as_str())?;
                let ddate_str = form
                    .get("death")
                    .ok_or(UploadError::user_facing("death is none"))?
                    .to_string();
                let death = naive_date_from_str(ddate_str.as_str())?;

                let carousel = crate::components::carousel::CarouselTemplate { images: vec![] };
                let ded = Dedication {
                    name: form
                        .get("fullname")
                        .ok_or(UploadError::user_facing("fullname is none"))?
                        .to_string(),
                    bio: form
                        .get("bio")
                        .ok_or(UploadError::user_facing("bio is none"))?
                        .to_string(),
                    birth,
                    death,
                    carousel,
                };
                Ok(Box::new(ded))
            }
        }
    }
}

#[tracing::instrument(name = "upload handler", skip(data))]
pub async fn upload_handler(
    Path(item_str): Path<String>,
    State(data): State<SharedState>,
    Form(form): Form<Value>,
) -> Result<impl IntoResponse, DataApiReturn> {
    let item = UploadItem::try_from(item_str.as_str()).expect("failed to get upload item");
    warn!("got item: {:?}", item);
    match item.form_into_(form) {
        Ok(uploadable) => {
            warn!("got uploadable: {:?}", uploadable);
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
    let response = Response::new("success".to_string());
    Ok(response)
}

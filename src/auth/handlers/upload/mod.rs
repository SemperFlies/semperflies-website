pub mod attachments;
pub mod form;
pub mod multipart;
pub use self::{attachments::*, form::*, multipart::*};

use crate::database::models::{
    DBAddressParams, DBDedicationParams, DBPatrolLogParams, DBResourceParams, DBTestimonialParams,
};
use anyhow::anyhow;
use chrono::NaiveDate;

#[derive(Debug)]
pub enum UploadItem {
    Address(DBAddressParams),
    Support(DBResourceParams),
    Debrief(DBTestimonialParams),
    PatrolLog(DBPatrolLogParams),
    Dedication(DBDedicationParams),
}

pub const IMAGES_DIRECTORY: &str = "public/assets/images";

pub trait UploadItemType<T>
where
    T: std::fmt::Debug,
{
    fn try_from_str(str: &str) -> anyhow::Result<Self>
    where
        Self: Sized;
    async fn into_item(self, form_or_multipart: T) -> anyhow::Result<UploadItem>;
}

pub(super) fn naive_date_from_str(str: &str) -> anyhow::Result<NaiveDate> {
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

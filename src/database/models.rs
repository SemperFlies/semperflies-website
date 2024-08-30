use anyhow::anyhow;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tracing::warn;
use uuid::Uuid;

use super::handles::DbData;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct DBImage {
    pub id: uuid::Uuid,
    pub path: String,
    pub alt: String,
    pub subtitle: Option<String>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct DBImageParams {
    pub path: String,
    pub alt: String,
    pub subtitle: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DBAddress {
    pub id: uuid::Uuid,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub line_1: String,
    pub line_2: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBAddressParams {
    pub city: String,
    pub state: String,
    pub zip: String,
    pub line_1: String,
    pub line_2: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DBDedication {
    pub id: uuid::Uuid,
    pub name: String,
    pub bio: String,
    pub birth: NaiveDate,
    pub death: NaiveDate,
    pub img_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBDedicationParams {
    pub name: String,
    pub bio: String,
    pub birth: NaiveDate,
    pub death: NaiveDate,
    pub img_params: Vec<DBImageParams>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DBPatrolLog {
    pub id: uuid::Uuid,
    pub heading: String,
    pub description: String,
    pub date: NaiveDate,
    pub img_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBPatrolLogParams {
    pub heading: String,
    pub description: String,
    pub date: NaiveDate,
    pub img_params: Vec<DBImageParams>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DBTestimonial {
    pub id: uuid::Uuid,
    pub firstname: String,
    pub lastname: String,
    pub bio: Option<String>,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBTestimonialParams {
    pub firstname: String,
    pub lastname: String,
    pub bio: Option<String>,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DBResource {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub missions: Vec<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website_url: Option<String>,
    pub address_id: Option<uuid::Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBResourceParams {
    pub name: String,
    pub description: String,
    pub missions: Vec<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website_url: Option<String>,
    pub address: Option<DBAddressParams>,
}

impl DbData<DBImageParams> for DBImage {
    fn id(&self) -> Uuid {
        self.id
    }
    fn table_name() -> String {
        "images".to_string()
    }
    fn fields() -> Vec<String> {
        vec!["path", "alt", "subtitle"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
    fn bind_tables(
        params: DBImageParams,
        query: super::handles::QueryType<Self>,
    ) -> super::handles::QueryType<Self> {
        query
            .bind(params.path)
            .bind(params.alt)
            .bind(params.subtitle)
    }
}

impl DbData<DBAddressParams> for DBAddress {
    fn id(&self) -> uuid::Uuid {
        self.id
    }
    fn table_name() -> String {
        "addresses".to_string()
    }
    fn fields() -> Vec<String> {
        vec!["city", "state", "zip", "line_1", "line_2"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
    fn bind_tables(
        params: DBAddressParams,
        query: super::handles::QueryType<Self>,
    ) -> super::handles::QueryType<Self> {
        query
            .bind(params.city)
            .bind(params.state)
            .bind(params.zip)
            .bind(params.line_1)
            .bind(params.line_2)
    }
}

impl DbData<DBDedicationParams> for DBDedication {
    fn id(&self) -> uuid::Uuid {
        self.id
    }
    fn table_name() -> String {
        "dedications".to_string()
    }
    fn fields() -> Vec<String> {
        vec!["name", "bio", "birth", "death", "img_ids"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
    fn take_images(params: &mut DBDedicationParams) -> Option<Vec<DBImageParams>> {
        Some(params.img_params.drain(..).collect())
    }

    fn bind_tables(
        params: DBDedicationParams,
        query: super::handles::QueryType<Self>,
    ) -> super::handles::QueryType<Self> {
        query
            .bind(params.name)
            .bind(params.bio)
            .bind(params.birth)
            .bind(params.death)
    }
}

impl DbData<DBPatrolLogParams> for DBPatrolLog {
    fn id(&self) -> uuid::Uuid {
        self.id
    }
    fn table_name() -> String {
        "patrol_logs".to_string()
    }
    fn fields() -> Vec<String> {
        vec!["heading", "description", "date", "img_ids"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
    fn take_images(params: &mut DBPatrolLogParams) -> Option<Vec<DBImageParams>> {
        Some(params.img_params.drain(..).collect())
    }
    fn bind_tables(
        params: DBPatrolLogParams,
        query: super::handles::QueryType<Self>,
    ) -> super::handles::QueryType<Self> {
        query
            .bind(params.heading)
            .bind(params.description)
            .bind(params.date)
    }
}

impl DbData<DBTestimonialParams> for DBTestimonial {
    fn id(&self) -> uuid::Uuid {
        self.id
    }
    fn table_name() -> String {
        "testimonials".to_string()
    }
    fn fields() -> Vec<String> {
        vec!["firstname", "lastname", "bio", "content"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }
    fn bind_tables(
        params: DBTestimonialParams,
        query: super::handles::QueryType<Self>,
    ) -> super::handles::QueryType<Self> {
        query
            .bind(params.firstname)
            .bind(params.lastname)
            .bind(params.bio)
            .bind(params.content)
    }
}

impl DbData<DBResourceParams> for DBResource {
    fn id(&self) -> uuid::Uuid {
        self.id
    }
    fn table_name() -> String {
        "support_resources".to_string()
    }
    fn fields() -> Vec<String> {
        vec![
            "name",
            "description",
            "missions",
            "phone",
            "email",
            // "address_id",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    fn bind_tables(
        params: DBResourceParams,
        query: super::handles::QueryType<Self>,
    ) -> super::handles::QueryType<Self> {
        query
            .bind(params.name)
            .bind(params.description)
            .bind(params.missions)
            .bind(params.phone)
            .bind(params.email)
        // .bind(params.address_id)
    }

    async fn insert_one(
        params: DBResourceParams,
        pool: &sqlx::Pool<sqlx::Postgres>,
    ) -> anyhow::Result<Self> {
        let mut id_opt = Option::<Uuid>::None;
        if let Some(add) = params.address {
            warn!("expects address: {:?}", add);
            let query = "
                SELECT id
                FROM addresses
                WHERE city = $1
                  AND state = $2
                  AND zip = $3
                  AND line_1 = $4
                  AND (line_2 IS DISTINCT FROM $5);
            ";

            id_opt = sqlx::query_scalar(query)
                .bind(add.city.to_owned())
                .bind(add.state.to_owned())
                .bind(add.zip.to_owned())
                .bind(add.line_1.to_owned())
                .bind(add.line_2.to_owned())
                .fetch_optional(pool)
                .await?;

            warn!("got id opt: {:?}", id_opt);

            if id_opt.is_none() {
                let dbadd = DBAddressParams {
                    city: add.city,
                    state: add.state,
                    zip: add.zip,
                    line_1: add.line_1,
                    line_2: add.line_2,
                };

                let add = DBAddress::insert_one(dbadd, pool).await?;
                id_opt = Some(add.id);
            }
        }

        let query = format!(
                    "INSERT INTO {} (name, description, missions, phone, email, address_id) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *;",
                    Self::table_name(),
                );
        let ret = sqlx::query_as::<_, Self>(&query)
            .bind(params.name)
            .bind(params.description)
            .bind(params.missions)
            .bind(params.phone)
            .bind(params.email)
            .bind(id_opt)
            .fetch_one(pool)
            .await?;
        Ok(ret)
    }

    async fn update_one(
        params: DBResourceParams,
        pool: &sqlx::Pool<sqlx::Postgres>,
        id: Uuid,
    ) -> anyhow::Result<()> {
        let mut id_opt = Option::<Uuid>::None;
        if let Some(add) = params.address {
            let query = "
                SELECT id
                FROM addresses
                WHERE city = $1
                  AND state = $2
                  AND zip = $3
                  AND line_1 = $4
                  AND (line_2 IS DISTINCT FROM $5)
            ";

            id_opt = sqlx::query_scalar(query)
                .bind(add.city.to_owned())
                .bind(add.state.to_owned())
                .bind(add.zip.to_owned())
                .bind(add.line_1.to_owned())
                .bind(add.line_2.to_owned())
                .fetch_one(pool)
                .await?;
            if id_opt.is_none() {
                let dbadd = DBAddressParams {
                    city: add.city,
                    state: add.state,
                    zip: add.zip,
                    line_1: add.line_1,
                    line_2: add.line_2,
                };

                let add = DBAddress::insert_one(dbadd, pool).await?;
                id_opt = Some(add.id);
            }
        }

        let query = format!(
                    "UPDATE {} WHERE id = $1 (name, description, missions, phone, email, address_id) VALUES ($2, $3, $4, $5, $6, $7) RETURNING *;",
                    Self::table_name(),
                );
        let ret = sqlx::query_as::<_, Self>(&query)
            .bind(id)
            .bind(params.name)
            .bind(params.description)
            .bind(params.missions)
            .bind(params.phone)
            .bind(params.email)
            .bind(id_opt)
            .fetch_one(pool)
            .await?;
        Ok(())
    }
}

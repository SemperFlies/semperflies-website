use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sqlx::{
    database::HasArguments,
    postgres::PgRow,
    query::{Query, QueryAs},
    Arguments, FromRow, Pool, Postgres, Row,
};
use tracing::warn;
use uuid::Uuid;

use crate::{auth::handlers::upload::UploadItem, database::models::DBImage};

use super::models::DBImageParams;

pub(super) type QueryType<'q, O> =
    QueryAs<'q, Postgres, O, <Postgres as HasArguments<'q>>::Arguments>;

pub trait DbData<P>:
    std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Send + Unpin + for<'r> FromRow<'r, PgRow>
where
    P: std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Send + Unpin,
{
    fn table_name() -> String;
    fn fields() -> Vec<String>;
    fn id(&self) -> Uuid;
    fn take_images(_params: &mut P) -> Option<Vec<DBImageParams>> {
        None
    }
    fn bind_tables(params: P, query: QueryType<Self>) -> QueryType<Self>;

    async fn get_single_by(pool: &Pool<Postgres>, id: Uuid) -> anyhow::Result<Option<Self>> {
        let query = format!("SELECT * FROM {} WHERE id = $1;", Self::table_name());
        let strct = sqlx::query_as::<_, Self>(&query)
            .bind(id)
            .fetch_optional(pool)
            .await?;
        Ok(strct)
    }

    async fn get_multiple(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Self>> {
        let query = format!("SELECT * FROM {};", Self::table_name());
        let all = sqlx::query_as::<_, Self>(&query).fetch_all(pool).await?;
        warn!("got all: {:?}", all);
        Ok(all)
    }

    async fn delete_one(&self, pool: &Pool<Postgres>) -> anyhow::Result<Option<Self>> {
        let query = format!("DELETE FROM {} WHERE id = $1;", Self::table_name());
        let strct = sqlx::query_as::<_, Self>(&query)
            .bind(self.id())
            .fetch_optional(pool)
            .await?;
        Ok(strct)
    }

    async fn delete_one_with_id(id: Uuid, pool: &Pool<Postgres>) -> anyhow::Result<()> {
        warn!("deleting item with id: {}", id);
        let query = format!(
            "DELETE FROM {} WHERE id = $1 RETURNING *;",
            Self::table_name()
        );
        if sqlx::query_as::<_, Self>(&query)
            .bind(id)
            .fetch_optional(pool)
            .await?
            .is_none()
        {
            warn!("nothing returned by deletion query");
            return Err(anyhow!(
                "problem with database deletion, no struct returned"
            ));
        }
        Ok(())
    }

    async fn delete_many(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Self>> {
        let query = format!("DELETE FROM {};", Self::table_name());
        let strct = sqlx::query_as::<_, Self>(&query).fetch_all(pool).await?;
        Ok(strct)
    }

    async fn insert_one(params: P, pool: &Pool<Postgres>) -> anyhow::Result<Self> {
        if Self::fields().contains(&"img_ids".to_string()) {
            warn!("make sure you've inserted images before inserting this'");
        }
        let binds = Self::fields()
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, _)| {
                let mut str_to_push = format!("${}", i + 1);
                if i + 1 != Self::fields().len() {
                    str_to_push.push_str(",");
                }
                acc.push_str(&str_to_push);
                acc
            });
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *;",
            Self::table_name(),
            Self::fields().join(","),
            binds
        );
        println!("running query: {}", query);
        let q = sqlx::query_as::<_, Self>(&query);
        let ret = Self::bind_tables(params, q).fetch_one(pool).await?;
        Ok(ret)
    }

    async fn update_one(params: P, pool: &Pool<Postgres>, id: Uuid) -> anyhow::Result<()> {
        let binds = Self::fields()
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, _)| {
                acc.push_str(&format!("${}", i + 1));
                acc
            });
        let query = format!(
            "UPDATE {} WHERE id = {} ({}) VALUES ({});",
            Self::table_name(),
            id,
            Self::fields().join(","),
            binds
        );
        let q = sqlx::query_as::<_, Self>(&query);
        Self::bind_tables(params, q).fetch_one(pool).await?;
        Ok(())
    }
}

impl DBImage {
    fn insert_query<D, P>() -> String
    where
        P: std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Send + Unpin,
        D: DbData<P>,
    {
        let binds = D::fields()
            .iter()
            .enumerate()
            .fold(String::new(), |mut acc, (i, _)| {
                let mut str_to_push = format!("${}", i + 1);
                if i + 1 != D::fields().len() {
                    str_to_push.push_str(",");
                }
                acc.push_str(&str_to_push);
                acc
            });
        let query = format!(
            "INSERT INTO {} ({}) VALUES ({}) RETURNING *;",
            D::table_name(),
            D::fields().join(","),
            binds
        );
        println!("query: {}", query);
        query
    }

    pub async fn insert_multiple_with_images<D, P>(
        pool: &Pool<Postgres>,
        multiple: Vec<P>,
    ) -> anyhow::Result<()>
    where
        P: std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Send + Unpin,
        D: DbData<P>,
    {
        for mut params in multiple {
            let mut imgs_ids = vec![];
            if let Some(images) = D::take_images(&mut params) {
                for img in images {
                    let i = DBImage::insert_one(img, pool).await?;
                    imgs_ids.push(i.id);
                }
            } else {
                return Err(anyhow!("should include images"));
            }
            let query = Self::insert_query::<D, P>();

            let q = sqlx::query_as::<_, D>(&query);
            D::bind_tables(params, q)
                .bind(imgs_ids)
                .fetch_all(pool)
                .await?;
        }
        Ok(())
    }

    pub async fn get_multiple_with_images<D, P>(
        pool: &Pool<Postgres>,
    ) -> anyhow::Result<Vec<(D, Vec<DBImage>)>>
    where
        P: std::fmt::Debug + Serialize + for<'de> Deserialize<'de> + Send + Unpin,
        D: DbData<P>,
    {
        let query = format!(
            r#"
        SELECT
            d.*, i.*
        FROM {} d
        LEFT JOIN LATERAL unnest(d.img_ids) AS img_id ON true
        LEFT JOIN {} i ON i.id = img_id::uuid
        ORDER BY d.id, i.id
        "#,
            D::table_name(),
            Self::table_name()
        );
        let rows = sqlx::query(&query)
            .fetch_all(pool)
            .await
            .expect("failed to query database");

        let mut result = Vec::new();
        let mut current_item: Option<(D, Vec<DBImage>)> = None;

        for row in rows {
            let item_id: Uuid = row.get("id");
            let expected_imgs_amt: usize = row
                .try_get::<Vec<Uuid>, _>("img_ids")
                .ok()
                .unwrap_or(vec![])
                .len();
            warn!("expecting {:?} images", expected_imgs_amt);

            if current_item.is_none()
                || (current_item.as_ref().unwrap().0.id() != item_id
                    && current_item.as_ref().unwrap().1.len() == expected_imgs_amt)
            {
                if let Some(item) = current_item {
                    result.push(item);
                }
                let new_item: D = D::from_row(&row)?;
                current_item = Some((new_item, Vec::new()));
            }

            if row.try_get::<String, _>("path").is_ok()
                && row.try_get::<String, _>("alt").is_ok()
                && row.try_get::<Option<String>, _>("subtitle").is_ok()
            {
                if let Ok(image_id) = row.try_get::<Uuid, _>("id") {
                    let image = DBImage {
                        id: image_id,
                        path: row.get("path"),
                        alt: row.get("alt"),
                        subtitle: row.get("subtitle"),
                    };
                    current_item.as_mut().unwrap().1.push(image);
                }
            }

            if let Some(item) = current_item.take() {
                if item.1.len() == expected_imgs_amt {
                    warn!("pushing item: {:?}", item);
                    result.push(item);
                } else {
                    warn!("item: {:?} not ready", item);
                    current_item = Some(item);
                }
            }
        }

        Ok(result)
    }
}

mod tests {
    use std::{str::FromStr, sync::LazyLock};

    use chrono::NaiveDate;
    use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
    use uuid::Uuid;

    use crate::{
        database::models::{
            DBAddress, DBAddressParams, DBDedication, DBDedicationParams, DBImage, DBImageParams,
            DBPatrolLog, DBPatrolLogParams, DBResource, DBResourceParams, DBTestimonial,
            DBTestimonialParams,
        },
        telemetry::{get_subscriber, init_subscriber},
        TRACING,
    };

    use super::DbData;

    async fn connect_to_database() -> Pool<Postgres> {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_PRIVATE_URL").expect("DATABASE_URL must be set");
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
        {
            Ok(pool) => {
                tracing::info!("✅Connection to the database is successful!");
                pool
            }
            Err(err) => {
                panic!("eror connecting: {:?}", err)
            }
        }
    }

    #[tokio::test]
    async fn crud_test() {
        LazyLock::force(&TRACING);

        let pool = connect_to_database().await;
        let mut td = test_data();

        DBResource::delete_many(&pool).await.unwrap();
        DBAddress::delete_many(&pool).await.unwrap();
        DBDedication::delete_many(&pool).await.unwrap();
        DBPatrolLog::delete_many(&pool).await.unwrap();
        DBImage::delete_many(&pool).await.unwrap();
        DBTestimonial::delete_many(&pool).await.unwrap();

        let deds_amt = td.dedications.len();
        DBImage::insert_multiple_with_images::<DBDedication, DBDedicationParams>(
            &pool,
            td.dedications,
        )
        .await
        .unwrap();

        let logs_amt = td.logs.len();
        DBImage::insert_multiple_with_images::<DBPatrolLog, DBPatrolLogParams>(&pool, td.logs)
            .await
            .unwrap();

        let testi_amt = td.testimonials.len();
        for params in td.testimonials {
            DBTestimonial::insert_one(params, &pool).await.unwrap();
        }

        let res_amt = td.resources.len();
        for params in td.resources {
            DBResource::insert_one(params, &pool).await.unwrap();
        }

        let all_deds_and_imgs =
            DBImage::get_multiple_with_images::<DBDedication, DBDedicationParams>(&pool)
                .await
                .unwrap();
        assert_eq!(all_deds_and_imgs.len(), deds_amt);

        let all_logs_and_imgs =
            DBImage::get_multiple_with_images::<DBPatrolLog, DBPatrolLogParams>(&pool)
                .await
                .unwrap();

        assert_eq!(all_logs_and_imgs.len(), deds_amt);

        let all = DBResource::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), res_amt);

        for (i, a) in all.into_iter().enumerate() {
            if i % 2 == 0 {
                a.delete_one(&pool).await.unwrap();
            } else {
                let id = Uuid::parse_str(&a.id.to_string()).unwrap();
                println!("id: {}", id);
                DBResource::delete_one_with_id(id, &pool).await.unwrap();
            }
        }

        let all = DBAddress::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), res_amt);

        for (i, a) in all.into_iter().enumerate() {
            if i % 2 == 0 {
                a.delete_one(&pool).await.unwrap();
            } else {
                let id = Uuid::parse_str(&a.id.to_string()).unwrap();
                println!("id: {}", id);
                DBAddress::delete_one_with_id(id, &pool).await.unwrap();
            }
        }

        let all = DBDedication::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), deds_amt);

        for (i, a) in all.into_iter().enumerate() {
            if i % 2 == 0 {
                a.delete_one(&pool).await.unwrap();
            } else {
                let id = Uuid::parse_str(&a.id.to_string()).unwrap();
                println!("id: {}", id);
                DBDedication::delete_one_with_id(id, &pool).await.unwrap();
            }
        }

        let all = DBPatrolLog::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), logs_amt);

        for (i, a) in all.into_iter().enumerate() {
            if i % 2 == 0 {
                a.delete_one(&pool).await.unwrap();
            } else {
                let id = Uuid::parse_str(&a.id.to_string()).unwrap();
                println!("id: {}", id);
                DBPatrolLog::delete_one_with_id(id, &pool).await.unwrap();
            }
        }

        let all = DBTestimonial::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), testi_amt);

        for (i, a) in all.into_iter().enumerate() {
            if i % 2 == 0 {
                a.delete_one(&pool).await.unwrap();
            } else {
                // let id = format!("{}", a.id);
                let id = Uuid::parse_str(&a.id.to_string()).unwrap();
                DBTestimonial::delete_one_with_id(id, &pool).await.unwrap();
            }
        }

        let all = DBAddress::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), 0);
        let all = DBDedication::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), 0);
        let all = DBPatrolLog::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), 0);
        let all = DBTestimonial::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), 0);
        let all = DBResource::get_multiple(&pool).await.unwrap();
        assert_eq!(all.len(), 0);
    }

    struct TestData {
        dedications: Vec<DBDedicationParams>,
        logs: Vec<DBPatrolLogParams>,
        testimonials: Vec<DBTestimonialParams>,
        resources: Vec<DBResourceParams>,
    }

    fn test_data() -> TestData {
        let images = vec![
            DBImageParams {
                path: "path1".to_string(),
                alt: "alt1".to_string(),
                subtitle: None,
            },
            DBImageParams {
                path: "path2".to_string(),
                alt: "alt2".to_string(),
                subtitle: None,
            },
            DBImageParams {
                path: "path3".to_string(),
                alt: "alt3".to_string(),
                subtitle: None,
            },
        ];

        let dedications = vec![DBDedicationParams {
            name: "John Doe".to_string(),
            bio: "A famous person".to_string(),
            birth: NaiveDate::from_ymd_opt(1980, 5, 15).unwrap(),
            death: NaiveDate::from_ymd_opt(2050, 12, 31).unwrap(),
            img_params: images.clone(),
        }];

        let logs = vec![DBPatrolLogParams {
            heading: "Patrol Log 1".to_string(),
            description: "This is a patrol log description".to_string(),
            date: NaiveDate::from_ymd_opt(2023, 4, 1).unwrap(),
            img_params: images.clone(),
        }];

        let testimonials = vec![DBTestimonialParams {
            firstname: "Jane".to_string(),
            lastname: "Smith".to_string(),
            bio: Some("A satisfied customer".to_string()),
            content: "I really enjoyed the service!".to_string(),
        }];

        let resources = vec![
            DBResourceParams {
                name: "Resource 1".to_string(),
                description: "This is a resource description".to_string(),
                missions: vec!["Mission 1".to_string(), "Mission 2".to_string()],
                phone: Some("555-1234".to_string()),
                email: Some("resource1@example.com".to_string()),
                address: Some(DBAddressParams {
                    city: "Los Angeles".to_string(),
                    state: "CA".to_string(),
                    zip: "90001".to_string(),
                    line_1: "456 Elm St".to_string(),
                    line_2: Some("Apt 2".to_string()),
                }),
            },
            DBResourceParams {
                name: "Resource 2".to_string(),
                description: "This is a resource 2 description".to_string(),
                missions: vec!["Mission 1".to_string(), "Mission 2".to_string()],
                phone: Some("999-1234".to_string()),
                email: Some("resource2@example.com".to_string()),
                address: Some(DBAddressParams {
                    city: "Carson".to_string(),
                    state: "CA".to_string(),
                    zip: "90745".to_string(),
                    line_1: "123 Main St".to_string(),
                    line_2: None,
                }),
            },
        ];

        TestData {
            // images,
            dedications,
            logs,
            testimonials,
            resources,
        }
    }
}

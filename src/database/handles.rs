use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use sqlx::{
    database::HasArguments,
    postgres::PgRow,
    query::{Query, QueryAs},
    Arguments, FromRow, Pool, Postgres,
};
use tracing::warn;
use uuid::Uuid;

use crate::auth::handlers::upload::UploadItem;

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

mod tests {
    use std::{str::FromStr, sync::LazyLock};

    use chrono::NaiveDate;
    use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
    use uuid::Uuid;

    use crate::{
        database::models::{
            DBAddress, DBAddressParams, DBDedication, DBDedicationParams, DBPatrolLog,
            DBPatrolLogParams, DBResource, DBResourceParams, DBTestimonial, DBTestimonialParams,
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
        let td = test_data();

        DBResource::delete_many(&pool).await.unwrap();
        DBAddress::delete_many(&pool).await.unwrap();
        DBAddress::delete_many(&pool).await.unwrap();
        DBDedication::delete_many(&pool).await.unwrap();
        DBPatrolLog::delete_many(&pool).await.unwrap();
        DBTestimonial::delete_many(&pool).await.unwrap();

        let deds_amt = td.dedications.len();
        for params in td.dedications {
            DBDedication::insert_one(params, &pool).await.unwrap();
        }
        let logs_amt = td.logs.len();
        for params in td.logs {
            DBPatrolLog::insert_one(params, &pool).await.unwrap();
        }
        let testi_amt = td.testimonials.len();
        for params in td.testimonials {
            DBTestimonial::insert_one(params, &pool).await.unwrap();
        }

        let res_amt = td.resources.len();
        for params in td.resources {
            DBResource::insert_one(params, &pool).await.unwrap();
        }

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
        // addresses: Vec<DBAddress>,
        dedications: Vec<DBDedicationParams>,
        logs: Vec<DBPatrolLogParams>,
        testimonials: Vec<DBTestimonialParams>,
        resources: Vec<DBResourceParams>,
    }

    fn test_data() -> TestData {
        let dedications = vec![DBDedicationParams {
            name: "John Doe".to_string(),
            bio: "A famous person".to_string(),
            birth: NaiveDate::from_ymd_opt(1980, 5, 15).unwrap(),
            death: NaiveDate::from_ymd_opt(2050, 12, 31).unwrap(),
            img_urls: vec!["https://example.com/image1.jpg".to_string()],
        }];

        let logs = vec![DBPatrolLogParams {
            heading: "Patrol Log 1".to_string(),
            description: "This is a patrol log description".to_string(),
            date: NaiveDate::from_ymd_opt(2023, 4, 1).unwrap(),
            img_urls: vec![],
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
            // addresses,
            dedications,
            logs,
            testimonials,
            resources,
        }
    }
}

use axum::Json;
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing::info;
use crate::models::images::UserStitches;
use crate::error::AppError;
use crate::models::user::{User, UserSignup};

#[derive(Clone)]
pub struct Store {
    pub conn_pool: PgPool,
}

pub async fn new_pool() -> PgPool {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap()
}

impl Store {
    pub fn with_pool(pool: PgPool) -> Self {
        Self { conn_pool: pool }
    }

    pub async fn test_database(&self) -> Result<(), sqlx::Error> {
        let row: (i64,) = sqlx::query_as("SELECT $1")
            .bind(150_i64)
            .fetch_one(&self.conn_pool)
            .await?;

        info!("{}", &row.0);

        assert_eq!(row.0, 150);
        Ok(())
    }

    pub async fn get_user(&self, email: &str) -> Result<User, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
                SELECT email, password, banned FROM users WHERE email ILIKE $1
            "#,
        )
            .bind(email)
            .fetch_one(&self.conn_pool)
            .await?;
        Ok(user)
    }

    pub async fn create_user(&self, user: UserSignup) -> Result<Json<Value>, AppError> {
        // TODO: Encrypt/bcrypt user passwords
        let result = sqlx::query("INSERT INTO users(email, password) values ($1, $2)")
            .bind(&user.email)
            .bind(&user.password)
            .execute(&self.conn_pool)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if result.rows_affected() < 1 {
            Err(AppError::InternalServerError)
        } else {
            Ok(Json(
                serde_json::json!({"message": "User created successfully!"}),
            ))
        }
    }

    pub async fn get_user_stitches_data(&self, user_email: &str) -> Vec<UserStitches> {
        let res = sqlx::query!(
            r#"
                SELECT id, search_terms, created_on FROM user_stitches
                WHERE user_id in (SELECT id FROM users WHERE email ILIKE $1)
                ORDER BY created_on DESC
            "#,
            user_email,
        )
            .fetch_all(&self.conn_pool)
            .await
            .unwrap();

        res.into_iter().map(|row | UserStitches {
            id: row.id,
            search_terms: row.search_terms,
            created_on: row.created_on.to_string()
        }).collect()
    }

    pub async fn get_user_stitches(&self, id: i32) -> Option<Vec<u8>> {
        let res = sqlx::query!(
            r#"
                SELECT image_data FROM user_stitches
                WHERE id = $1
            "#,
            id
        )
            .fetch_one(&self.conn_pool)
            .await
            .unwrap();

        res.image_data
    }

    pub async fn delete_user_stitches(&self, user_stitches_id: i32, user_email: &str) -> Option<String> {
        println!("delete started {:?}", &user_email);
        println!("delete started {:?}", user_stitches_id);

        let _ = sqlx::query(
            r#"
                DELETE FROM user_stitches WHERE id = $1 and user_id in
                (SELECT id FROM users WHERE email ILIKE $2)
            "#)
            .bind(user_stitches_id)
            .bind(user_email)
            .execute(&self.conn_pool).await;

        Option::from("".to_string())
    }

    pub async fn get_cache(&self, api_type: &str, url: &str) -> Option<String> {
        let res = sqlx::query!(
            r#"
                SELECT json_results FROM cache
                WHERE api_type = $1 AND url = $2
            "#,
            api_type,
            url
        )
            .fetch_one(&self.conn_pool)
            .await
            .unwrap();

        res.json_results
    }

    pub async fn get_users(&self) -> Vec<User> {
        let res = sqlx::query!(
            r#"
                SELECT email, password, banned FROM users
                WHERE admin_access IS NULL or admin_access = false
                ORDER BY email ASC
            "#
        )
            .fetch_all(&self.conn_pool)
            .await
            .unwrap();

        res.into_iter().map(|row | User {
            email: row.email,
            password: "".to_string(),
            banned: Option::from(row.banned)
        }).collect()
    }

    pub async fn check_admin_access(&self, email: &str) -> bool {
        let res = sqlx::query!(
            r#"
                SELECT id FROM users
                WHERE email ILIKE $1 AND admin_access = true
            "#,
            email
        )
            .fetch_all(&self.conn_pool)
            .await
            .unwrap();

        if res.len() > 0 {
            return true;
        }
        false
    }

    pub async fn has_cache(&self, api_type: &str, url: &str) -> bool {
        let res = sqlx::query!(
            r#"
                SELECT id FROM cache
                WHERE api_type = $1 AND url = $2
            "#,
            api_type,
            url
        )
            .fetch_all(&self.conn_pool)
            .await
            .unwrap();

        if res.len() > 0 {
            return true;
        }
        false
    }

    pub async fn save_cache(&self, api_type: &str, url: &str, search_terms: &str, json_results: &str) -> Result<(), sqlx::Error> {
        let _res = sqlx::query!(
            r#"
                INSERT INTO cache (api_type, url, search_terms, json_results)
                VALUES ($1, $2, $3, $4)
                RETURNING *
            "#,
            api_type,
            url,
            search_terms,
            json_results,
        )
            .fetch_one(&self.conn_pool)
            .await?;

        Ok(())
    }

    pub async fn ban_user(&self, admin_email: &str, user_emails: Vec<String>) -> Result<(), sqlx::Error> {

        if self.check_admin_access(admin_email).await {
            // reset banned access
            let _res = sqlx::query(
                r#"
                UPDATE users SET banned = FALSE
               "#
            ).execute(&self.conn_pool).await;


             for email in &user_emails {
                let _res = sqlx::query(
                    r#"
                        UPDATE users SET banned = TRUE where email ILIKE $1
                       "#
                )
                    .bind(&email)
                    .execute(&self.conn_pool).await;

            }
        }

        Ok(())
    }


    pub async fn save_user_stitches(&self, image_data: Vec<u8>, search_terms: &str, user_email: &str) -> Result<(), sqlx::Error> {
        let query_user = sqlx::query!(
            r#"
                SELECT id FROM users WHERE email ILIKE $1
            "#,
            user_email
        ).fetch_one(&self.conn_pool).await?;

        let _res = sqlx::query!(
            r#"
                INSERT INTO user_stitches (user_id, search_terms, image_data)
                VALUES ($1, $2, $3)
                RETURNING *
            "#,
            query_user.id,
            search_terms,
            image_data,
        )
            .fetch_one(&self.conn_pool)
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

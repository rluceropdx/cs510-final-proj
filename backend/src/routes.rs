use axum::response::Response;
use axum::routing::*;
use axum::Router;
use http::StatusCode;
use hyper::Body;
use sqlx::PgPool;
use tracing::info;

use crate::db::Store;
use crate::handlers::*;
use crate::{layers};

pub async fn app(pool: PgPool) -> Router {
    let db = Store::with_pool(pool);

    info!("Seeded database");

    let (cors_layer, trace_layer) = layers::get_layers();

    Router::new()
        // The router matches these FROM TOP TO BOTTOM explicitly!
        .route("/", get(login_screen))
        .route("/search", get(search_empty))
        .route("/search/:topic", get(search))
        .route("/display_stitch", get(display_last_stitched_image))
        .route("/display_stitch/:id", get(display_user_stitched_image))
        .route("/display_saved_stitches", get(display_saved_stitches))
        .route("/delete_stitch/:id", post(delete_stitch))
        .route("/save", post(save_images))
        .route("/users", post(register_user))
        .route("/manage_users", get(manage_users))
        .route("/update_users", post(update_users))
        .route("/logout", get(logout))
        .route("/login", post(validate_login))
        .route("/protected", get(protected))
        .route("/*_", get(handle_404))
        .layer(cors_layer)
        .layer(trace_layer)
        .with_state(db)
}

async fn handle_404() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("The requested page could not be found"))
        .unwrap()
}

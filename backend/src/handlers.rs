use std::fs;
use std::fs::{File};
use std::io::Write;
use axum::extract::{Path, State};
use axum::Json;
use axum::response::{Html, Response};
use http::header::{LOCATION, SET_COOKIE};
use http::{HeaderValue, StatusCode};
use hyper::Body;
use jsonwebtoken::Header;
use tera::Context;
use tracing::error;

use reqwest::{Client};
use serde_json::{Value};
use serde_json::Value::Null;
use crate::models::images::{Item, NasaImages, UserStitches};

use crate::db::Store;
use crate::template::TEMPLATES;

use stitchy_core::{ImageFiles, OrderBy, TakeFrom, Stitch, AlignmentMode};
use crate::error::AppError;
use crate::get_timestamp_after_8_hours;
use crate::models::user::{Claims, User, UserSignup, KEYS};

//#[derive(Display)]
pub enum Data {
    Text(String),
    Boolean(bool),
    Number(i32)
}

pub async fn login_screen(
    State(_proj_database): State<Store>,
) -> Result<Html<String>, AppError>  {

    let mut context = Context::new();
    context.insert("name", "Farani Lucero");

    let rendered = TEMPLATES
        .render("login.html", &context)
        .unwrap_or_else(|err| {
            error!("Template rendering error: {}", err);
            panic!()
        });

    Ok(Html(rendered))
}

pub async fn search_empty() -> Html<String> {
    let context = Context::new();

    let rendered = TEMPLATES
        .render("redirect.html", &context)
        .unwrap_or_else(|err| {
            error!("Template rendering error: {}", err);
            panic!()
        });
    Html(rendered)
}

pub async fn display_last_stitched_image(
    State(_proj_database): State<Store>,
) -> Response<Body> {
    // Replace "path/to/image.jpg" with the actual path to your image file
    let file_path = "temp/stitched_image.jpg";

    if std::path::Path::new(file_path).exists() {
        // Read the file into a byte vector
        let file_content = tokio::fs::read(file_path).await.unwrap();

        // Create a response with the file content and appropriate headers
        Response::builder()
            .status(StatusCode::FOUND)
            .header("content-type", "image/jpeg")
            .body(Body::from(file_content))
            .unwrap()
    } else {
        // File not found response
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap()
    }
}

pub async fn display_user_stitched_image(
    Path(id): Path<i32>, // localhost:3000/search/apollo moon landing
    State(proj_database): State<Store>,
) -> Response<Body> {

    let file_content = proj_database.get_user_stitches(id).await.unwrap();

    // File found response
    Response::builder()
        .status(StatusCode::FOUND)
        .body(Body::from(file_content))
        .unwrap()
}

pub async fn display_saved_stitches(
    claims: Claims,
    State(proj_database): State<Store>,
) -> Html<String> {

    let data_array: Vec<UserStitches> = proj_database.get_user_stitches_data(&claims.email).await;

    let mut context = Context::new();
    context.insert("name", "Farani Lucero");
    context.insert("data_array", &data_array);

    if claims.email.len() > 0 {
        context.insert("logged_in", &true);
        context.insert("user_email", &claims.email);
    } else {
        context.insert("logged_in", &false);
        context.insert("user_email", "");
    }

    let rendered = TEMPLATES
        .render("user_stitches.html", &context)
        .unwrap_or_else(|err| {
            error!("Template rendering error: {}", err);
            panic!()
        });
    Html(rendered)
}

pub async fn save_images (
    claims: Claims,
    State(proj_database): State<Store>,
    post_params: String,
) -> String {
    // empty temp folder
    let dir_path = "temp";
    for entry in fs::read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
    
        if path.is_file() {
            fs::remove_file(&path).expect("TODO: panic message");
            println!("Deleted file: {:?}", path);
        }
    }

    let params_str: Value = serde_json::from_str(&post_params).unwrap();
    let search_terms: String = serde_json::from_str(&params_str.get("search_terms").unwrap().to_string()).unwrap();
    let image_sources: Vec<String> = serde_json::from_str(&params_str.get("image_sources").unwrap().to_string()).unwrap();
    let mut counter = 0;
    for image in image_sources {
        counter += 1;

        // Send a GET request to the image URL
        let client = Client::new();
        let response:Result<reqwest::Response, reqwest::Error> = client.get(&image)
            .send()
            .await;

        // Check if the request was successful
        match response {
            Ok(response) => {
                // Check if the response status is 200 OK
                if response.status().is_success() {
                    // Read the response content (image data) into a buffer
                    let buffer: Vec<u8> = response.bytes().await.unwrap().to_vec();

                    let filename = "temp/image".to_string() + &*counter.to_string() + &*".jpg".to_string();
                    // Save the image data to a local file
                    let mut file = File::create(&filename).expect("Failed to create file.");
                    file.write_all(&buffer).expect("Failed to write to file.");

                    println!("Image downloaded and saved as {}", &filename);
                } else {
                    println!("Failed to download image: Status code {}", response.status());
                }
            }
            Err(err) => {
                println!("Failed to send request: {}", err);
            }
        }
    }

    let image_files = ImageFiles::builder()
        .add_current_directory(vec!["temp"]).unwrap()
        .build().unwrap()
        .sort_and_truncate_by(counter, OrderBy::Alphabetic, TakeFrom::Start, false).unwrap();

    let file_path = "temp/stitched_image.jpg";

    let _stitch = Stitch::builder()
        .image_files(image_files).unwrap()
        .width_limit(1000)
        .alignment(AlignmentMode::Horizontal)
        .stitch()
        .unwrap()
        .save(file_path);

    if std::path::Path::new(file_path).exists() {
        // Read the file into a byte vector
        let file_content = tokio::fs::read(file_path).await.unwrap();
        let _ = proj_database.save_user_stitches(file_content, &search_terms, &claims.email).await;
    }

    "saved".to_string()
}

pub async fn search (
    claims: Claims,
    Path(topic): Path<String>, // localhost:3000/search/apollo moon landing
    State(proj_database): State<Store>,
) -> Html<String> {

    let mut cache_exists: bool = false;
    let mut json_response: Value = Null;
    let url: String = build_search_url(&topic).await;

    if !proj_database.has_cache("nasa", &url).await {
        println!("no cache exists for {:?}", &topic);
        let response = query_nasa(&url).await.unwrap();

        if response.status().is_success() {
            json_response = response.json().await.unwrap();
            let escaped_str = json_response.to_string().replace("'", "''");

            let _ = proj_database.save_cache("nasa", &url, &topic, &escaped_str).await;
        }
        else {
            println!("Request failed with status code: {:?}", response.status());
        }
    } else {
        println!("cache found for {:?}", &topic);
        cache_exists = true;
        json_response = proj_database.get_cache("nasa", &url).await.unwrap().parse().unwrap();
    }

    parse_nasa_response(&json_response, &topic, &cache_exists, &claims).await
}

async fn parse_nasa_response(json_response: &Value, topic: &str, cache_exists: &bool, claims: &Claims) -> Html<String> {
    let nasa_images: NasaImages = serde_json::from_str(&json_response.to_string()).unwrap();
    let image_data: Vec<Item> = nasa_images.collection.items;

    let mut context = Context::new();
    context.insert("name", "Farani Lucero");
    context.insert("images", &image_data);
    context.insert("search_content", &topic);
    context.insert("cache_exists", cache_exists);

    if claims.email.len() > 0 {
        context.insert("logged_in", &true);
        context.insert("user_email", &claims.email);
    } else {
        context.insert("logged_in", &false);
        context.insert("user_email", "");
    }

    let rendered = TEMPLATES
        .render("index.html", &context)
        .unwrap_or_else(|err| {
            error!("Template rendering error: {}", err);
            panic!()
        });
    Html(rendered)
}

async fn build_search_url(topic: &str) -> String {
    // let nasa_api_key = std::env::var("NASA_API_KEY").unwrap();
    // api key not needed for NASA image and video api
    let mut search_string = "https://images-api.nasa.gov/search?q=".to_string();
    search_string.push_str(topic);
    search_string.push_str("&description=");
    search_string.push_str(topic);
    search_string.push_str("&media_type=image");

    search_string
}

async fn query_nasa(search_url: &str) -> Result<reqwest::Response, reqwest::Error> {

    // Create a reqwest client
    let client = Client::new();
    let response:Result<reqwest::Response, reqwest::Error> = client.get(search_url).send().await;
    return response;
}

pub async fn delete_stitch(
    claims: Claims,
    State(proj_database): State<Store>,
    Path(id): Path<i32>, // localhost:3000/delete_stitch/10
) -> Response<Body> {
    println!("stitch_id {:?}", id);

    if Some(&claims).is_some() {
        println!("email {:?}", &claims.email);
        let _ = proj_database.delete_user_stitches(id, &claims.email).await.unwrap();
    }
    // File not found response
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap()

}


pub async fn register_user(
    State(proj_database): State<Store>,
    Json(mut credentials): Json<UserSignup>,
) -> Result<Json<Value>, AppError> {
    // We should also check to validate other things at some point like email address being in right format

    if credentials.email.is_empty() || credentials.password.is_empty() {
        return Err(AppError::MissingCredentials);
    }

    if credentials.password != credentials.confirm_password {
        return Err(AppError::MissingCredentials);
    }

    // Check to see if there is already a user in the database with the given email address
    let existing_user = proj_database.get_user(&credentials.email).await;

    if let Ok(_) = existing_user {
        return Err(AppError::UserAlreadyExists);
    }

    // Here we're assured that our credentials are valid and the user doesn't already exist
    // hash their password
    let hash_config = argon2::Config::default();
    let salt = std::env::var("SALT").expect("Missing SALT");
    let hashed_password = match argon2::hash_encoded(
        credentials.password.as_bytes(),
        // If you'd like unique salts per-user, simply pass &[] and argon will generate them for you
        salt.as_bytes(),
        &hash_config,
    ) {
        Ok(result) => result,
        Err(_) => {
            return Err(AppError::Any(anyhow::anyhow!("Password hashing failed")));
        }
    };

    credentials.password = hashed_password;

    let new_user = proj_database.create_user(credentials).await?;
    Ok(new_user)
}

pub async fn validate_login(
    State(proj_database): State<Store>,
    Json(creds): Json<User>,
) -> Result<Response<Body>, AppError> {

    if creds.email.is_empty() || creds.password.is_empty() {
        return Err(AppError::MissingCredentials);
    }

    let existing_user = proj_database.get_user(&creds.email).await?;

    let is_password_correct =
        match argon2::verify_encoded(&*existing_user.password, creds.password.as_bytes()) {
            Ok(result) => result,
            Err(_) => {
                return Err(AppError::InternalServerError);
            }
        };

    if !is_password_correct {
        return Err(AppError::InvalidPassword);
    }

    // at this point we've authenticated the user's identity
    // create JWT to return
    let claims = Claims {
        id: 0,
        email: creds.email.to_owned(),
        exp: get_timestamp_after_8_hours(),
    };

    let token = jsonwebtoken::encode(&Header::default(), &claims, &KEYS.encoding)
        .map_err(|_| AppError::MissingCredentials)?;

    let cookie = cookie::Cookie::build("jwt", token).http_only(true).finish();

    let mut response = Response::builder()
        .status(StatusCode::FOUND)
        .body(Body::empty())
        .unwrap();

    let headers = response.headers_mut();
    headers.insert(LOCATION, HeaderValue::from_static("/"));
    headers.insert(SET_COOKIE, HeaderValue::from_str(&cookie.to_string()).unwrap());

    Ok(response)
}


pub async fn logout(
) -> Result<Response<Body>, AppError> {

    let mut response = Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "/")
        .body(Body::empty())
        .unwrap();

    let delete_cookie_header = format!("{}=; Max-Age=0", "jwt");
    let header_value = HeaderValue::from_str(&delete_cookie_header).unwrap();
    response.headers_mut().insert(SET_COOKIE, header_value);

    Ok(response)
}

pub async fn protected(claims: Claims) -> Result<String, AppError> {
    Ok(format!(
        "Welcome to the PROTECTED area :) \n Your claim data is: {}",
        claims
    ))
}




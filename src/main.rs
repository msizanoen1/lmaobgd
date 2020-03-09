#[macro_use]
extern crate diesel;

mod actions;
mod models;
mod schema;

use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use std::collections::HashMap;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

use actix_web::{get, post, web, App, HttpResponse, HttpServer};

#[post("/upload")]
async fn api_upload(
    pool: web::Data<DbPool>,
    web::Json(json): web::Json<models::JsApiUpload>,
) -> Result<HttpResponse, actix_web::Error> {
    let conn = web::block(move || pool.get()).await?;
    web::block(move || actions::js_upload_call(&conn, json)).await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/data")]
async fn api_data(
    pool: web::Data<DbPool>,
) -> Result<web::Json<HashMap<i32, i32>>, actix_web::Error> {
    let db = web::block(move || pool.get()).await?;
    let data = web::block(move || actions::js_get_data(&db)).await?;
    Ok(web::Json(data))
}

fn api() -> actix_web::Scope {
    web::scope("/api").service(api_data).service(api_upload)
}

#[actix_rt::main]
async fn main() -> Result<(), exitfailure::ExitFailure> {
    env_logger::init();
    let _ = dotenv::dotenv();

    let db = std::env::var("DATABASE_URL")?;
    let cm = ConnectionManager::new(&db);
    let pool = DbPool::builder().build(cm)?;

    HttpServer::new(move || App::new().data(pool.clone()).service(api()))
        .bind("0.0.0.0:5000")?
        .run()
        .await?;
    Ok(())
}

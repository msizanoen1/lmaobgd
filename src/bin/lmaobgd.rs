use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{get, middleware, post, web, App, FromRequest, HttpResponse, HttpServer};
use actix_web_httpauth::extractors::basic::BasicAuth;
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use lmaobgd::{actions, models};
use log::info;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
#[derive(Clone)]
struct RoDbPool(DbPool);

#[post("/upload")]
async fn api_upload(
    pool: web::Data<DbPool>,
    auth: BasicAuth,
    web::Json(json): web::Json<models::JsApiUpload>,
) -> Result<HttpResponse, actix_web::Error> {
    let db = web::block(move || pool.get()).await?;
    let db = Arc::new(Mutex::new(db));
    let key = auth.user_id().clone();
    let db2 = Arc::clone(&db);
    let (id, note) = web::block(move || actions::check_api_key(&db2.lock().unwrap(), &key))
        .await?
        .ok_or_else(|| HttpResponse::Unauthorized())?;
    info!(
        "api access id={} note={}",
        id,
        note.as_deref().unwrap_or("")
    );
    web::block(move || actions::upload_call(&db.lock().unwrap(), json)).await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/data")]
async fn api_data(
    pool: web::Data<RoDbPool>,
    auth: BasicAuth,
) -> Result<web::Json<HashMap<i32, i32>>, actix_web::Error> {
    let db = web::block(move || pool.0.get()).await?;
    let db = Arc::new(Mutex::new(db));
    let db2 = Arc::clone(&db);
    let key = auth.user_id().clone();
    let (id, note) = web::block(move || actions::check_api_key_r(&db2.lock().unwrap(), &key))
        .await?
        .ok_or_else(|| HttpResponse::Unauthorized())?;
    info!(
        "api access id={} note={}",
        id,
        note.as_deref().unwrap_or("")
    );
    let data = web::block(move || actions::get_data(&db.lock().unwrap())).await?;
    Ok(web::Json(data))
}

#[get("/check")]
async fn api_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[post("/set_reviewed")]
async fn api_set_reviewed(
    pool: web::Data<DbPool>,
    web::Json(ids): web::Json<Vec<i32>>,
    auth: BasicAuth,
) -> Result<HttpResponse, actix_web::Error> {
    let db = web::block(move || pool.get()).await?;
    let db = Arc::new(Mutex::new(db));
    let key = auth.user_id().clone();
    let db2 = Arc::clone(&db);
    let (id, note) = web::block(move || actions::check_api_key(&db2.lock().unwrap(), &key))
        .await?
        .ok_or_else(|| HttpResponse::Unauthorized())?;
    info!(
        "api access id={} note={}",
        id,
        note.as_deref().unwrap_or("")
    );
    web::block(move || actions::set_reviewed(&db.lock().unwrap(), &ids)).await?;
    Ok(HttpResponse::Ok().finish())
}

fn api() -> actix_web::Scope {
    web::scope("/api")
        .service(api_data)
        .service(api_upload)
        .service(api_set_reviewed)
        .service(api_check)
}

fn cors() -> actix_cors::CorsFactory {
    Cors::new()
        .allowed_methods(vec!["GET", "POST"])
        .allowed_headers(vec![header::CONTENT_TYPE, header::AUTHORIZATION])
        .finish()
}

/// LmaoBGD Web Service
#[derive(StructOpt)]
struct Args {
    /// Address to bind to
    #[structopt(short, long, default_value = "0.0.0.0:5000")]
    bind: SocketAddr,
    /// Database URL to use
    #[structopt(short, long, env, hide_env_values = true)]
    database_url: String,
    /// Read-only database URL to use
    #[structopt(long, env, hide_env_values = true)]
    database_url_ro: Option<String>,
    /// Writable database connection pool size
    #[structopt(long, default_value = "10")]
    db_writable_pool_size: u32,
    /// Read-only database connection pool size
    #[structopt(long, default_value = "10")]
    db_read_only_pool_size: u32,
}

#[actix_rt::main]
async fn main() -> Result<(), exitfailure::ExitFailure> {
    env_logger::init();
    let _ = dotenv::dotenv();
    let args = Args::from_args();

    let cm = ConnectionManager::new(&args.database_url);
    let pool = DbPool::builder()
        .max_size(args.db_writable_pool_size)
        .build(cm)?;
    let pool_ro = if let Some(url) = args.database_url_ro {
        let cm = ConnectionManager::new(&url);
        DbPool::builder()
            .max_size(args.db_read_only_pool_size)
            .build(cm)?
    } else {
        pool.clone()
    };

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(RoDbPool(pool_ro.clone()))
            .app_data(web::Json::<models::JsApiUpload>::configure(|cfg| {
                cfg.limit(128 * 1024 * 1024)
            }))
            .service(api())
            .wrap(cors())
            .wrap(middleware::Logger::default())
    })
    .bind(&args.bind)?
    .run()
    .await?;
    Ok(())
}

use actix_cors::Cors;
use actix_web::http::header;
use actix_web::{get, middleware, post, web, App, FromRequest, HttpResponse, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use lmaobgd::{actions, models};
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[derive(Copy, Clone)]
struct DoAuth(bool);

#[derive(Deserialize)]
struct Auth {
    key: Option<String>,
}

#[post("/upload")]
async fn api_upload(
    pool: web::Data<DbPool>,
    web::Query(auth): web::Query<Auth>,
    do_auth: web::Data<DoAuth>,
    web::Json(json): web::Json<models::JsApiUpload>,
) -> Result<HttpResponse, actix_web::Error> {
    let db = web::block(move || pool.get()).await?;
    let db = Arc::new(Mutex::new(db));
    if do_auth.0 {
        let key = auth.key.ok_or_else(|| HttpResponse::BadRequest())?;
        let db = Arc::clone(&db);
        let result = web::block(move || actions::check_api_key(&db.lock().unwrap(), &key)).await?;
        if !result {
            return Err(HttpResponse::Forbidden().into());
        }
    }
    web::block(move || actions::upload_call(&db.lock().unwrap(), json)).await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/data")]
async fn api_data(
    pool: web::Data<DbPool>,
) -> Result<web::Json<HashMap<i32, i32>>, actix_web::Error> {
    let db = web::block(move || pool.get()).await?;
    let data = web::block(move || actions::get_data(&db)).await?;
    Ok(web::Json(data))
}

#[post("/set_reviewed")]
async fn api_set_reviewed(
    pool: web::Data<DbPool>,
    web::Json(ids): web::Json<Vec<i32>>,
    web::Query(auth): web::Query<Auth>,
    do_auth: web::Data<DoAuth>,
) -> Result<HttpResponse, actix_web::Error> {
    let db = web::block(move || pool.get()).await?;
    let db = Arc::new(Mutex::new(db));
    if do_auth.0 {
        let key = auth.key.ok_or_else(|| HttpResponse::BadRequest())?;
        let db = Arc::clone(&db);
        let result = web::block(move || actions::check_api_key(&db.lock().unwrap(), &key)).await?;
        if !result {
            return Err(HttpResponse::Forbidden().into());
        }
    }
    web::block(move || actions::set_reviewed(&db.lock().unwrap(), &ids)).await?;
    Ok(HttpResponse::Ok().finish())
}

fn api() -> actix_web::Scope {
    web::scope("/api")
        .service(api_data)
        .service(api_upload)
        .service(api_set_reviewed)
}

fn cors() -> actix_cors::CorsFactory {
    Cors::new()
        .allowed_methods(vec!["GET", "POST"])
        .allowed_header(header::CONTENT_TYPE)
        .finish()
}

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long, default_value = "0.0.0.0:5000")]
    bind: SocketAddr,
    #[structopt(short = "a", long)]
    do_auth: bool,
}

#[actix_rt::main]
async fn main() -> Result<(), exitfailure::ExitFailure> {
    env_logger::init();
    let _ = dotenv::dotenv();
    let args = Args::from_args();

    let db = std::env::var("DATABASE_URL")?;
    let cm = ConnectionManager::new(&db);
    let pool = DbPool::builder().build(cm)?;
    let do_auth = args.do_auth;

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(DoAuth(do_auth))
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

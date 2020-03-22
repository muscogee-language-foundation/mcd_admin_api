#[macro_use]
extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenv::dotenv;
use std::env;

use actix_web::error::BlockingError;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

mod models;
mod schema;

use self::models::*;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/api/entries")]
async fn get_entries(pool: web::Data<DbPool>) -> impl Responder {
    use self::schema::entries::dsl::*;
    let conn = pool.get().expect("couldn't get db connection from pool");

    let query = entries.load::<Entry>(&conn);

    let results: Result<Vec<Entry>, BlockingError<diesel::result::Error>> =
        web::block(move || query).await;

    match results {
        Ok(words) => HttpResponse::Ok().json(words),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/api/entries/{id}")]
async fn get_entry(pool: web::Data<DbPool>, entry_id: web::Path<i32>) -> impl Responder {
    use self::schema::entries::dsl::*;
    let conn = pool.get().expect("couldn't get db connection from pool");

    let query = entries
        .filter(id.eq::<i32>(entry_id.to_string().parse().unwrap()))
        .first::<Entry>(&conn);

    let results: Result<Entry, BlockingError<diesel::result::Error>> =
        web::block(move || query).await;

    match results {
        Ok(word) => HttpResponse::Ok().json(word),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

fn connection_string() -> String {
    dotenv().ok();

    let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL should be set");
    connection_string
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let connection = connection_string();
    let manager = ConnectionManager::<PgConnection>::new(connection);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(get_entries)
            .service(get_entry)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

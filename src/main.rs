#[macro_use]
extern crate diesel;
extern crate bcrypt;
extern crate dotenv;

use actix_cors::Cors;
use actix_web::error::BlockingError;
use actix_web::middleware::Logger;
use actix_web::{delete, get, post, put, web, App, HttpResponse, HttpServer, Responder};
use bcrypt::verify;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel::{delete as delete_entry, insert_into, update as update_entry};
use jsonwebtoken::{encode, EncodingKey, Header};
use utils::{connection_pool, connection_string, secret_string, THIRTY_DAYS_IN_MS};

mod middleware;
mod models;
mod schema;
mod utils;

use crate::models::*;
use crate::schema::entries::dsl::{id as e_id, *};
use crate::schema::users::dsl::*;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[get("/api/entries")]
async fn get_entries(pool: web::Data<DbPool>) -> impl Responder {
    let conn = connection_pool(pool);

    let query = entries.load::<Entry>(&conn);

    let results: Result<Vec<Entry>, BlockingError<diesel::result::Error>> =
        web::block(move || query).await;

    match results {
        Ok(words) => HttpResponse::Ok().json(words),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/api/entries")]
async fn create_entry(pool: web::Data<DbPool>, form: web::Form<EntryFormData>) -> impl Responder {
    let conn = connection_pool(pool);

    let insert = insert_into(entries).values(form.clone()).execute(&conn);

    let results = web::block(move || insert).await;

    match results {
        Ok(_) => HttpResponse::Created().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[get("/api/entries/{id}")]
async fn get_entry(pool: web::Data<DbPool>, entry_id: web::Path<i32>) -> impl Responder {
    let conn = connection_pool(pool);

    let query = entries
        .filter(e_id.eq::<i32>(entry_id.to_string().parse().unwrap()))
        .first::<Entry>(&conn);

    let result: Result<Entry, BlockingError<diesel::result::Error>> =
        web::block(move || query).await;

    match result {
        Ok(word) => HttpResponse::Ok().json(word),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[put("/api/entries/{id}")]
async fn change_entry(
    pool: web::Data<DbPool>,
    entry_id: web::Path<i32>,
    form: web::Form<EntryFormData>,
) -> impl Responder {
    let conn = connection_pool(pool);
    let request =
        update_entry(entries.filter(e_id.eq::<i32>(entry_id.to_string().parse().unwrap())))
            .set(form.clone())
            .execute(&conn);

    let results = web::block(move || request).await;

    match results {
        Ok(_) => HttpResponse::NoContent().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}
#[delete("/api/entries/{id}")]
async fn remove_entry(pool: web::Data<DbPool>, entry_id: web::Path<i32>) -> impl Responder {
    let conn = connection_pool(pool);

    let request =
        delete_entry(entries.filter(e_id.eq::<i32>(entry_id.to_string().parse().unwrap())))
            .execute(&conn);

    let results = web::block(move || request).await;

    match results {
        Ok(_) => HttpResponse::NoContent().finish(),
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/api/login")]
async fn login(pool: web::Data<DbPool>, form: web::Form<LoginFormData>) -> impl Responder {
    let conn = connection_pool(pool);
    let secret = secret_string();

    let query = users
        .filter(email.eq::<String>(form.email.to_string().parse().unwrap()))
        .first::<User>(&conn);

    let result = web::block(move || query).await;

    match result {
        Ok(user) => {
            let validate = verify(&form.password, &user.password);

            match validate {
                Ok(_) => {
                    let claims = Claims {
                        sub: user.email.clone(),
                        exp: THIRTY_DAYS_IN_MS,
                    };
                    let token = encode(
                        &Header::default(),
                        &claims,
                        &EncodingKey::from_secret(secret.as_ref()),
                    )
                    .unwrap();

                    let response: LoginResponse = LoginResponse {
                        token,
                        email: user.email.clone(),
                    };

                    HttpResponse::Ok().json(response)
                }
                _ => HttpResponse::Unauthorized().finish(),
            }
        }
        _ => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let connection = connection_string();
    let manager = ConnectionManager::<PgConnection>::new(connection);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(middleware::Authorization)
            .wrap(
                Cors::new()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET"])
                    .max_age(3600)
                    .finish(),
            )
            .data(pool.clone())
            .service(get_entries)
            .service(get_entry)
            .service(create_entry)
            .service(remove_entry)
            .service(change_entry)
            .service(login)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

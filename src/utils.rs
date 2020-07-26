extern crate dotenv;

use actix_web::web;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use dotenv::dotenv;
use std::env;

use crate::DbPool;

pub const THIRTY_DAYS_IN_MS: usize = 2592000000;

pub fn connection_string() -> String {
    dotenv().ok();

    let is_release = match env::var("PROFILE") {
        Ok(profile) => profile == "release",
        _ => false,
    };

    let db_var = if is_release {
        "PROD_DATABASE_URL"
    } else {
        "DEV_DATABASE_URL"
    };

    return env::var(db_var).expect("DATABASE_URL should be set");
}

pub fn secret_string() -> String {
    dotenv().ok();

    return env::var("SECRET").expect("SECRET should be set");
}

pub fn connection_pool(
    pool: web::Data<DbPool>,
) -> PooledConnection<ConnectionManager<PgConnection>> {
    pool.get().expect("couldn't get db connection from pool")
}

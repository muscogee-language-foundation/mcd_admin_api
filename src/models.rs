use serde::{Deserialize, Serialize};

use crate::schema::*;

#[derive(Debug, Clone, Queryable, Serialize, Deserialize)]
pub struct Entry {
    pub id: i32,
    pub creek: String,
    pub english: String,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Queryable, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, Queryable, Serialize, Deserialize)]
pub struct LoginFormData {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub email: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Insertable, AsChangeset)]
#[table_name = "entries"]
pub struct EntryFormData {
    pub creek: String,
    pub english: String,
    pub tags: Option<String>,
}

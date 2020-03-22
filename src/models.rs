use serde::{Deserialize, Serialize};
#[derive(Queryable, Serialize, Deserialize)]
pub struct Entry {
  pub id: i32,
  pub creek: String,
  pub english: String,
  pub tags: Option<String>,
}

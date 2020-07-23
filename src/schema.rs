table! {
    entries (id) {
        id -> Int4,
        creek -> Varchar,
        english -> Varchar,
        tags -> Nullable<Varchar>,
    }
}

table! {
    users (id) {
        id -> Int4,
        email -> Varchar,
        password -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    entries,
    users,
);

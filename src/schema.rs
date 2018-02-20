table! {
    listings (id) {
        id -> Integer,
        domain -> Varchar,
    }
}

table! {
    requests (id) {
        id -> Integer,
        publisher -> Varchar,
        userquality -> Nullable<Integer>,
    }
}

table! {
    responses (id) {
        id -> Integer,
        publisher -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(listings, requests, responses,);

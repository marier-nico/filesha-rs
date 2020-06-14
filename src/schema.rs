table! {
    shares (link) {
        link -> Text,
        path -> Text,
    }
}

table! {
    users (id) {
        id -> Integer,
        email -> Text,
        display_name -> Text,
        password -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    shares,
    users,
);

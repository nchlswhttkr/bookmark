table! {
    bookmark (id) {
        id -> Integer,
        url -> Text,
        name -> Text,
        created -> Timestamp,
    }
}

table! {
    tag (id) {
        id -> Integer,
        bookmark_id -> Integer,
        value -> Text,
    }
}

joinable!(tag -> bookmark (bookmark_id));

allow_tables_to_appear_in_same_query!(
    bookmark,
    tag,
);

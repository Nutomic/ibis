// @generated automatically by Diesel CLI.

diesel::table! {
    article (id) {
        id -> Int4,
        title -> Text,
        text -> Text,
        #[max_length = 255]
        ap_id -> Varchar,
        #[max_length = 255]
        instance_id -> Varchar,
        latest_version -> Text,
        local -> Bool,
    }
}

diesel::table! {
    edit (id) {
        id -> Int4,
        #[max_length = 255]
        ap_id -> Varchar,
        diff -> Text,
        article_id -> Int4,
        version -> Text,
        local -> Bool,
    }
}

diesel::joinable!(edit -> article (article_id));

diesel::allow_tables_to_appear_in_same_query!(article, edit,);

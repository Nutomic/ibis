// @generated automatically by Diesel CLI.

diesel::table! {
    article (id) {
        id -> Int4,
        title -> Text,
        text -> Text,
        #[max_length = 255]
        ap_id -> Varchar,
        instance_id -> Int4,
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
        previous_version -> Text,
    }
}

diesel::table! {
    instance (id) {
        id -> Int4,
        #[max_length = 255]
        ap_id -> Varchar,
        inbox_url -> Text,
        #[max_length = 255]
        articles_url -> Varchar,
        public_key -> Text,
        private_key -> Nullable<Text>,
        last_refreshed_at -> Timestamptz,
        local -> Bool,
    }
}

diesel::table! {
    instance_follow (id) {
        id -> Int4,
        follower_id -> Int4,
        followed_id -> Int4,
        pending -> Bool,
    }
}

diesel::joinable!(article -> instance (instance_id));
diesel::joinable!(edit -> article (article_id));

diesel::allow_tables_to_appear_in_same_query!(article, edit, instance, instance_follow,);

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
        protected -> Bool,
        approved -> Bool,
        published -> Timestamptz,
    }
}

diesel::table! {
    comment (id) {
        id -> Int4,
        creator_id -> Int4,
        article_id -> Int4,
        parent_id -> Nullable<Int4>,
        content -> Text,
        depth -> Int4,
        #[max_length = 255]
        ap_id -> Varchar,
        local -> Bool,
        deleted -> Bool,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    conflict (id) {
        id -> Int4,
        hash -> Uuid,
        diff -> Text,
        summary -> Text,
        creator_id -> Int4,
        article_id -> Int4,
        previous_version_id -> Uuid,
        published -> Timestamptz,
    }
}

diesel::table! {
    edit (id) {
        id -> Int4,
        creator_id -> Int4,
        hash -> Uuid,
        #[max_length = 255]
        ap_id -> Varchar,
        diff -> Text,
        summary -> Text,
        article_id -> Int4,
        previous_version_id -> Uuid,
        published -> Timestamptz,
        pending -> Bool,
    }
}

diesel::table! {
    instance (id) {
        id -> Int4,
        domain -> Text,
        #[max_length = 255]
        ap_id -> Varchar,
        topic -> Nullable<Text>,
        #[max_length = 255]
        articles_url -> Nullable<Varchar>,
        #[max_length = 255]
        inbox_url -> Varchar,
        public_key -> Text,
        private_key -> Nullable<Text>,
        last_refreshed_at -> Timestamptz,
        local -> Bool,
        #[max_length = 255]
        instances_url -> Nullable<Varchar>,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    instance_follow (id) {
        id -> Int4,
        instance_id -> Int4,
        follower_id -> Int4,
        pending -> Bool,
    }
}

diesel::table! {
    instance_stats (id) {
        id -> Int4,
        users -> Int4,
        users_active_month -> Int4,
        users_active_half_year -> Int4,
        articles -> Int4,
        comments -> Int4,
    }
}

diesel::table! {
    jwt_secret (id) {
        id -> Int4,
        secret -> Varchar,
    }
}

diesel::table! {
    local_user (id) {
        id -> Int4,
        password_encrypted -> Text,
        person_id -> Int4,
        admin -> Bool,
    }
}

diesel::table! {
    person (id) {
        id -> Int4,
        username -> Text,
        #[max_length = 255]
        ap_id -> Varchar,
        #[max_length = 255]
        inbox_url -> Varchar,
        public_key -> Text,
        private_key -> Nullable<Text>,
        last_refreshed_at -> Timestamptz,
        local -> Bool,
        #[max_length = 20]
        display_name -> Nullable<Varchar>,
        #[max_length = 1000]
        bio -> Nullable<Varchar>,
    }
}

diesel::joinable!(article -> instance (instance_id));
diesel::joinable!(comment -> article (article_id));
diesel::joinable!(comment -> person (creator_id));
diesel::joinable!(conflict -> article (article_id));
diesel::joinable!(conflict -> person (creator_id));
diesel::joinable!(edit -> article (article_id));
diesel::joinable!(edit -> person (creator_id));
diesel::joinable!(instance_follow -> instance (instance_id));
diesel::joinable!(instance_follow -> person (follower_id));
diesel::joinable!(local_user -> person (person_id));

diesel::allow_tables_to_appear_in_same_query!(
    article,
    comment,
    conflict,
    edit,
    instance,
    instance_follow,
    instance_stats,
    jwt_secret,
    local_user,
    person,
);

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
        published -> Timestamptz,
        removed -> Bool,
        updated -> Timestamptz,
        pending -> Bool,
    }
}

diesel::table! {
    article_follow (local_user_id, article_id) {
        local_user_id -> Int4,
        article_id -> Int4,
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
    email_verification (id) {
        id -> Int4,
        local_user_id -> Int4,
        email -> Text,
        verification_token -> Text,
        published -> Timestamptz,
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
        instances_url -> Varchar,
        name -> Nullable<Text>,
    }
}

diesel::table! {
    instance_follow (instance_id, follower_id) {
        instance_id -> Int4,
        follower_id -> Int4,
        pending -> Bool,
    }
}

diesel::table! {
    instance_stats (id) {
        users -> Int4,
        users_active_month -> Int4,
        users_active_half_year -> Int4,
        articles -> Int4,
        comments -> Int4,
        id -> Int4,
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
        password_encrypted -> Nullable<Text>,
        person_id -> Int4,
        admin -> Bool,
        email -> Nullable<Text>,
        email_verified -> Bool,
        email_notifications -> Bool,
    }
}

diesel::table! {
    notification (id) {
        id -> Int4,
        local_user_id -> Int4,
        article_id -> Int4,
        creator_id -> Int4,
        comment_id -> Nullable<Int4>,
        edit_id -> Nullable<Int4>,
        published -> Timestamptz,
        conflict_id -> Nullable<Int4>,
    }
}

diesel::table! {
    oauth_account (oauth_issuer_url, local_user_id) {
        local_user_id -> Int4,
        oauth_issuer_url -> Text,
        oauth_user_id -> Text,
        published -> Timestamptz,
        updated -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    password_reset_request (id) {
        id -> Int4,
        local_user_id -> Int4,
        token -> Text,
        published -> Timestamptz,
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

diesel::table! {
    person_follow (person_id, follower_id) {
        person_id -> Int4,
        follower_id -> Int4,
    }
}

diesel::table! {
    sent_activity (id) {
        #[max_length = 255]
        id -> Varchar,
        json -> Text,
        published -> Timestamptz,
    }
}

diesel::joinable!(article -> instance (instance_id));
diesel::joinable!(article_follow -> article (article_id));
diesel::joinable!(article_follow -> local_user (local_user_id));
diesel::joinable!(comment -> article (article_id));
diesel::joinable!(comment -> person (creator_id));
diesel::joinable!(conflict -> article (article_id));
diesel::joinable!(conflict -> person (creator_id));
diesel::joinable!(edit -> article (article_id));
diesel::joinable!(edit -> person (creator_id));
diesel::joinable!(email_verification -> local_user (local_user_id));
diesel::joinable!(instance_follow -> instance (instance_id));
diesel::joinable!(instance_follow -> person (follower_id));
diesel::joinable!(local_user -> person (person_id));
diesel::joinable!(notification -> article (article_id));
diesel::joinable!(notification -> comment (comment_id));
diesel::joinable!(notification -> conflict (conflict_id));
diesel::joinable!(notification -> edit (edit_id));
diesel::joinable!(notification -> local_user (local_user_id));
diesel::joinable!(notification -> person (creator_id));
diesel::joinable!(oauth_account -> local_user (local_user_id));
diesel::joinable!(password_reset_request -> local_user (local_user_id));

diesel::allow_tables_to_appear_in_same_query!(
    article,
    article_follow,
    comment,
    conflict,
    edit,
    email_verification,
    instance,
    instance_follow,
    instance_stats,
    jwt_secret,
    local_user,
    notification,
    oauth_account,
    password_reset_request,
    person,
    person_follow,
    sent_activity,
);

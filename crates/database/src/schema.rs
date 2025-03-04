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
    }
}

diesel::table! {
    article_follow (local_user_id, article_id) {
        local_user_id -> Int4,
        article_id -> Int4,
    }
}

diesel::table! {
    category (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
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
    community (id) {
        id -> Int4,
        #[max_length = 20]
        name -> Varchar,
        #[max_length = 100]
        title -> Varchar,
        description -> Nullable<Text>,
        category_id -> Int4,
        creator_id -> Int4,
        removed -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    community_follower (id) {
        id -> Int4,
        community_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    community_moderator (id) {
        id -> Int4,
        community_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    community_user_ban (id) {
        id -> Int4,
        community_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
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
    post (id) {
        id -> Int4,
        #[max_length = 100]
        name -> Varchar,
        url -> Nullable<Text>,
        body -> Nullable<Text>,
        creator_id -> Int4,
        community_id -> Int4,
        removed -> Bool,
        locked -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    post_like (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        score -> Int2,
        published -> Timestamp,
    }
}

diesel::table! {
    post_read (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    post_saved (id) {
        id -> Int4,
        post_id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::table! {
    site (id) {
        id -> Int4,
        #[max_length = 20]
        name -> Varchar,
        description -> Nullable<Text>,
        creator_id -> Int4,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_ (id) {
        id -> Int4,
        #[max_length = 20]
        name -> Varchar,
        #[max_length = 40]
        fedi_name -> Varchar,
        #[max_length = 20]
        preferred_username -> Nullable<Varchar>,
        password_encrypted -> Text,
        email -> Nullable<Text>,
        icon -> Nullable<Bytea>,
        admin -> Bool,
        banned -> Bool,
        published -> Timestamp,
        updated -> Nullable<Timestamp>,
    }
}

diesel::table! {
    user_ban (id) {
        id -> Int4,
        user_id -> Int4,
        published -> Timestamp,
    }
}

diesel::joinable!(article -> instance (instance_id));
diesel::joinable!(article_follow -> article (article_id));
diesel::joinable!(article_follow -> local_user (local_user_id));
diesel::joinable!(comment -> article (article_id));
diesel::joinable!(comment -> person (creator_id));
diesel::joinable!(community -> category (category_id));
diesel::joinable!(community -> user_ (creator_id));
diesel::joinable!(community_follower -> community (community_id));
diesel::joinable!(community_follower -> user_ (user_id));
diesel::joinable!(community_moderator -> community (community_id));
diesel::joinable!(community_moderator -> user_ (user_id));
diesel::joinable!(community_user_ban -> community (community_id));
diesel::joinable!(community_user_ban -> user_ (user_id));
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
diesel::joinable!(post -> community (community_id));
diesel::joinable!(post -> user_ (creator_id));
diesel::joinable!(post_like -> post (post_id));
diesel::joinable!(post_like -> user_ (user_id));
diesel::joinable!(post_read -> post (post_id));
diesel::joinable!(post_read -> user_ (user_id));
diesel::joinable!(post_saved -> post (post_id));
diesel::joinable!(post_saved -> user_ (user_id));
diesel::joinable!(site -> user_ (creator_id));
diesel::joinable!(user_ban -> user_ (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    article,
    article_follow,
    category,
    comment,
    community,
    community_follower,
    community_moderator,
    community_user_ban,
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
    person,
    post,
    post_like,
    post_read,
    post_saved,
    site,
    user_,
    user_ban,
);

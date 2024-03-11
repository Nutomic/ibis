create table instance (
    id serial primary key,
    domain text not null unique,
    ap_id varchar(255) not null unique,
    description text,
    articles_url varchar(255) not null unique,
    inbox_url varchar(255) not null,
    public_key text not null,
    private_key text,
    last_refreshed_at timestamptz not null default now(),
    local bool not null
);

create table person (
    id serial primary key,
    username text not null,
    ap_id varchar(255) not null unique,
    inbox_url varchar(255) not null,
    public_key text not null,
    private_key text,
    last_refreshed_at timestamptz not null default now(),
    local bool not null
);

create table local_user (
    id serial primary key,
    password_encrypted text not null,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    admin bool not null
);

create table instance_follow (
    id serial primary key,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    follower_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    pending boolean not null,
    unique(instance_id, follower_id)
);

create table article (
    id serial primary key,
    title text not null,
    text text not null,
    ap_id varchar(255) not null unique,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    local bool not null,
    protected bool not null
);

create table edit (
    id serial primary key,
    creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    hash uuid not null,
    ap_id varchar(255) not null unique,
    diff text not null,
    summary text not null,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    previous_version_id uuid not null,
    created timestamptz not null
);

create table conflict (
    id serial primary key,
    hash uuid not null,
    diff text not null,
    summary text not null,
    creator_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    previous_version_id uuid not null
);

-- generate a jwt secret
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE jwt_secret (
    id serial PRIMARY KEY,
    secret varchar NOT NULL DEFAULT gen_random_uuid ()
);

INSERT INTO jwt_secret DEFAULT VALUES;

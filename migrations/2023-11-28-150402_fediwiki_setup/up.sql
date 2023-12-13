create table instance (
    id serial primary key,
    ap_id varchar(255) not null unique,
    inbox_url text not null,
    articles_url varchar(255) not null unique,
    public_key text not null,
    private_key text,
    last_refreshed_at timestamptz not null default now(),
    local bool not null
);

create table person (
    id serial primary key,
    username text not null,
    ap_id varchar(255) not null unique,
    inbox_url text not null,
    public_key text not null,
    private_key text,
    last_refreshed_at timestamptz not null default now(),
    local bool not null
);

create table local_user (
    id serial primary key,
    password_encrypted text not null,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL
);

create table instance_follow (
    id serial primary key,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    follower_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    pending boolean not null,
    unique(instance_id, follower_id)
);

create table article (
    id serial primary key,
    title text not null,
    text text not null,
    ap_id varchar(255) not null unique,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    local bool not null
);

create table edit (
    id serial primary key,
    hash uuid not null,
    ap_id varchar(255) not null unique,
    diff text not null,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    previous_version_id uuid not null
);

create table conflict (
    id uuid primary key,
    diff text not null,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    previous_version_id uuid not null
);
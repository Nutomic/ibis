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

create table instance_follow (
    id serial primary key,
    follower_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    followed_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    pending boolean not null

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
    ap_id varchar(255) not null unique,
    diff text not null,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    version text not null,
    previous_version text not null
)
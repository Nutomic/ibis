create table article (
    id serial primary key,
    title text not null,
    text text not null,
    ap_id varchar(255) not null unique,
    instance_id varchar(255) not null,
    local bool not null
);

create table edit (
    id serial primary key,
    ap_id varchar(255) not null unique,
    diff text not null,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    version text not null,
    previous_version text not null,
    local bool not null
)
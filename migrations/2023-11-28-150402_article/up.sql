create table article (
    id serial primary key,
    title text not null,
    text text not null,
    ap_id varchar(255) not null,
    instance_id varchar(255) not null,
    latest_version text not null,
    local bool not null
);

create table edit (
    id serial primary key,
    ap_id varchar(255) not null,
    diff text not null,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    version text not null,
    local bool not null
)
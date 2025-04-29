create table sent_activity (
    id varchar(255) primary key,
    json text not null,
    published timestamptz not null default now()
);

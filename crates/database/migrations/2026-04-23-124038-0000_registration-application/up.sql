create table registration_application (
    id serial primary key,
    local_user_id int not null references local_user on delete cascade on update cascade,
    answer text not null,
    published_at timestamptz not null default now()
);
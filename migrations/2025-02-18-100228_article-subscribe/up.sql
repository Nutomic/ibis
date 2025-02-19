create table article_follow(
    local_user_id int references local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int references article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, 
    primary key(local_user_id, article_id));

create table article_notification(
    id serial primary key,
    local_user_id int references local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int references article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    new_comments bool not null default false,
    new_edits bool not null default false,
    published timestamptz NOT NULL DEFAULT now(),
    unique(local_user_id, article_id, new_comments, new_edits),
    check(new_comments != new_edits)
);

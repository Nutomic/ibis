create table article_follow(
    local_user_id int not null references local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int not null references article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, 
    primary key(local_user_id, article_id));

create table article_notification(
    id serial primary key,
    local_user_id int references local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int references article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int references comment ON UPDATE CASCADE ON DELETE CASCADE,
    edit_id int references edit ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz NOT NULL DEFAULT now(),
    unique(local_user_id, article_id, comment_id, edit_id),
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (comment_id, edit_id) = 1)
);

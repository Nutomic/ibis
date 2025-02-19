create table article_follow(
    person_id int not null references person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int not null references article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, 
    primary key(person_id, article_id));

create table article_notification(
    id serial primary key,
    person_id int references person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    comment_id int references person ON UPDATE CASCADE ON DELETE CASCADE,
    edit_id int references person ON UPDATE CASCADE ON DELETE CASCADE,
    -- Make sure only one of the columns is not null
    CHECK (num_nonnulls (comment_id, edit_id) = 1)
);

/*
-- TODO: test these triggers, read notifications from new table
CREATE FUNCTION create_comment_trigger()
RETURNS trigger AS $$
declare follow record;
BEGIN
  for follow in select * from article_follow as af where NEW.article_id = af.article_id
  loop
    insert into article_notifications(person_id, comment_id) values (follow.person_id, NEW.id);
  end loop;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_comment
BEFORE INSERT ON comment
FOR EACH ROW
EXECUTE PROCEDURE create_comment_trigger();

CREATE FUNCTION create_edit_trigger()
RETURNS trigger AS $$
declare follow record;
BEGIN
  for follow in select * from article_follow as af where NEW.article_id = af.article_id
  loop
    insert into article_notifications(person_id, edit_id) values (follow.person_id, NEW.id);
  end loop;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER create_edit
BEFORE INSERT ON edit
FOR EACH ROW
EXECUTE PROCEDURE create_edit_trigger();
*/
alter table notification drop constraint notification_check;
alter table notification add constraint notification_check CHECK (num_nonnulls (comment_id, edit_id) < 2);
alter table notification add column conflict_id int references conflict ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE notification ADD CONSTRAINT notification_local_user_id_article_id_conflict_id_key UNIQUE (local_user_id, article_id, conflict_id);
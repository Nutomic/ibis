alter table notification drop constraint notification_check;
alter table notification add constraint notification_check CHECK (num_nonnulls (comment_id, edit_id) = 1);
alter table notification drop column conflict_id;
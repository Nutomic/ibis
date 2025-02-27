alter table notification drop constraint notification_check;
alter table notification add constraint notification_check CHECK (num_nonnulls (comment_id, edit_id) < 2);
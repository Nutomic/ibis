
alter table instance_stats drop column id;
alter table instance_stats add column id serial primary key;
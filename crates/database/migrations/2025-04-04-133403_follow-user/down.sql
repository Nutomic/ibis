DROP TABLE person_follow;

alter table instance_follow drop constraint instance_follow_pkey;
alter table instance_follow add column id serial primary key;
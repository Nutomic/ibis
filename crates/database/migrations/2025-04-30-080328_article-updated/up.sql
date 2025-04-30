alter table article add column updated timestamptz not null default now();
alter table article add column pending bool not null default false;
alter table article add column updated timestamptz not null default now();

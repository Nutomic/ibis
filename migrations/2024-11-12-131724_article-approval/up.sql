alter table article add column approved bool not null default true;

alter table article add column published timestamptz not null default now();

alter table conflict add column published timestamptz not null default now();

alter table edit rename column created to published;
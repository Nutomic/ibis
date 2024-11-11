alter table instance add column instances_url varchar(255) unique;

alter table instance alter column articles_url drop not null;

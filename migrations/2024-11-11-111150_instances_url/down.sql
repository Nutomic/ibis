alter table instance drop column instances_url;

alter table instance alter column articles_url set not null;

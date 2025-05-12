update instance set instances_url = ap_id where instances_url is null;
alter table instance alter column instances_url set not null;
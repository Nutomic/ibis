alter table article drop column approved;

alter table article drop column published;

alter table conflict drop column published;

alter table edit rename column published to created;
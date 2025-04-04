CREATE TABLE person_follow (
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    follower_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    primary key (person_id, follower_id)
);

alter table instance_follow drop column id;
alter table instance_follow add primary key (instance_id, follower_id);
create table article_follow(
    person_id int not null references person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int not null references article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL, 
    primary key(person_id, article_id));
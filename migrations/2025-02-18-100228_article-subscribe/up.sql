create table article_follow(
    person_id int not null,
    article_id int not null, 
    primary key(person_id, article_id));
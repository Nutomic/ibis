ALTER TABLE local_user
    ALTER COLUMN password_encrypted SET NOT NULL;
alter table local_user drop column email;
    
drop table oauth_account;
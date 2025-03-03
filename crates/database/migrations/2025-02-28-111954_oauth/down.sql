ALTER TABLE local_user
    ALTER COLUMN password_encrypted SET NOT NULL;
alter table local_user drop column email;
alter table local_user drop column email_verified;
    
drop table oauth_account;
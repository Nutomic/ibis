ALTER TABLE local_user
    ALTER COLUMN password_encrypted SET NOT NULL;
alter table local_user drop column email;
alter table local_user drop column email_verified;
    
drop table oauth_account;
drop table email_verification;
alter table local_user drop column email_notifications;

drop table password_reset_request;
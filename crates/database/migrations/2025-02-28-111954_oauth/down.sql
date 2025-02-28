ALTER TABLE local_user
    ALTER COLUMN password_encrypted SET NOT NULL;
    
drop table oauth_account;
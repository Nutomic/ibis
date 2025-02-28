ALTER TABLE local_user
    ALTER COLUMN password_encrypted DROP NOT NULL;
    
CREATE TABLE oauth_account (
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    oauth_issuer_url text NOT NULL,
    oauth_user_id text NOT NULL,
    published timestamp with time zone DEFAULT now() NOT NULL,
    updated timestamp with time zone,
    UNIQUE (oauth_issuer_url, oauth_user_id),
    PRIMARY KEY (oauth_issuer_url, local_user_id)
);
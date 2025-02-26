CREATE TABLE instance (
    id serial PRIMARY KEY,
    domain text NOT NULL UNIQUE,
    ap_id varchar(255) NOT NULL UNIQUE,
    description text,
    articles_url varchar(255) NOT NULL UNIQUE,
    inbox_url varchar(255) NOT NULL,
    public_key text NOT NULL,
    private_key text,
    last_refreshed_at timestamptz NOT NULL DEFAULT now(),
    local bool NOT NULL
);

CREATE TABLE person (
    id serial PRIMARY KEY,
    username text NOT NULL,
    ap_id varchar(255) NOT NULL UNIQUE,
    inbox_url varchar(255) NOT NULL,
    public_key text NOT NULL,
    private_key text,
    last_refreshed_at timestamptz NOT NULL DEFAULT now(),
    local bool NOT NULL
);

CREATE TABLE local_user (
    id serial PRIMARY KEY,
    password_encrypted text NOT NULL,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    admin bool NOT NULL
);

CREATE TABLE instance_follow (
    id serial PRIMARY KEY,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    follower_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    pending boolean NOT NULL,
    UNIQUE (instance_id, follower_id)
);

CREATE TABLE article (
    id serial PRIMARY KEY,
    title text NOT NULL,
    text text NOT NULL,
    ap_id varchar(255) NOT NULL UNIQUE,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    local bool NOT NULL,
    protected bool NOT NULL
);

CREATE TABLE edit (
    id serial PRIMARY KEY,
    creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    hash uuid NOT NULL,
    ap_id varchar(255) NOT NULL UNIQUE,
    diff text NOT NULL,
    summary text NOT NULL,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    previous_version_id uuid NOT NULL,
    created timestamptz NOT NULL
);

CREATE TABLE CONFLICT (
    id serial PRIMARY KEY,
    hash uuid NOT NULL,
    diff text NOT NULL,
    summary text NOT NULL,
    creator_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    previous_version_id uuid NOT NULL
);

-- generate a jwt secret
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE jwt_secret (
    id serial PRIMARY KEY,
    secret varchar NOT NULL DEFAULT gen_random_uuid ()
);

INSERT INTO jwt_secret DEFAULT VALUES;

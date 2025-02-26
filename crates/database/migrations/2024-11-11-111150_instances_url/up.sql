ALTER TABLE instance
    ADD COLUMN instances_url varchar(255) UNIQUE;

ALTER TABLE instance
    ALTER COLUMN articles_url DROP NOT NULL;


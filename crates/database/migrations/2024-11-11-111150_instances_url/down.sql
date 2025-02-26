ALTER TABLE instance
    DROP COLUMN instances_url;

ALTER TABLE instance
    ALTER COLUMN articles_url SET NOT NULL;


ALTER TABLE article
    DROP COLUMN approved;

ALTER TABLE article
    DROP COLUMN published;

ALTER TABLE CONFLICT
    DROP COLUMN published;

ALTER TABLE edit RENAME COLUMN published TO created;


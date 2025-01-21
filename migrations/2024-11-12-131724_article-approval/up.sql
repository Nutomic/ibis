ALTER TABLE article
    ADD COLUMN approved bool NOT NULL DEFAULT TRUE;

ALTER TABLE article
    ADD COLUMN published timestamptz NOT NULL DEFAULT now();

ALTER TABLE CONFLICT
    ADD COLUMN published timestamptz NOT NULL DEFAULT now();

ALTER TABLE edit RENAME COLUMN created TO published;


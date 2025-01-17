CREATE OR REPLACE FUNCTION generate_unique_comment_id ()
    RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'http://example.com/' || string_agg(substr('abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz0123456789', ceil(random() * 62)::integer, 1), '')
    FROM
        generate_series(1, 20)
$$;

CREATE TABLE comment (
    id serial PRIMARY KEY,
    creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    article_id int REFERENCES article ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    parent_id int REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE,
    content text NOT NULL,
    depth int not null,
    ap_id varchar(255) NOT NULL UNIQUE DEFAULT generate_unique_comment_id(),
    local boolean NOT NULL,
    deleted boolean NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz
);

ALTER TABLE instance_stats
    ADD COLUMN comments int NOT NULL DEFAULT 0;

CREATE FUNCTION instance_stats_comment_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        instance_stats
    SET
        comments = comments + 1;
    RETURN NULL;
END
$$;

CREATE FUNCTION instance_stats_comment_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        instance_stats ia
    SET
        comments = comments - 1;
    RETURN NULL;
END
$$;

CREATE TRIGGER instance_stats_comment_insert
    AFTER INSERT ON comment
    FOR EACH ROW
    WHEN (NEW.local = TRUE)
    EXECUTE PROCEDURE instance_stats_comment_insert ();

CREATE TRIGGER instance_stats_comment_delete
    AFTER DELETE ON comment
    FOR EACH ROW
    WHEN (OLD.local = TRUE)
    EXECUTE PROCEDURE instance_stats_comment_delete ();

CREATE OR REPLACE FUNCTION instance_stats_activity (i text)
    RETURNS int
    LANGUAGE plpgsql
    AS $$
DECLARE
    count_ integer;
BEGIN
    SELECT
        count(a) INTO count_
    FROM (
        SELECT
            e.creator_id
        FROM
            edit e
            INNER JOIN person p ON e.creator_id = p.id
        WHERE
            e.published > ('now'::timestamp - i::interval)
            AND p.local = TRUE
        UNION
        SELECT
            c.creator_id
        FROM
            comment c
            INNER JOIN person p ON p.creator_id = c.id
        WHERE
            c.published > ('now'::timestamp - i::interval)
            AND p.local = TRUE) a;
    RETURN count_;
END;
$$;


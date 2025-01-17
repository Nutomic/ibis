DROP TRIGGER instance_stats_comment_insert ON comment;

DROP TRIGGER instance_stats_comment_delete ON comment;

DROP FUNCTION instance_stats_comment_insert;

DROP FUNCTION instance_stats_comment_delete;

ALTER TABLE instance_stats
    DROP COLUMN comments;

DROP TABLE comment;

DROP FUNCTION generate_unique_comment_id;

CREATE OR REPLACE FUNCTION instance_stats_activity (i text)
    RETURNS int
    LANGUAGE plpgsql
    AS $$
DECLARE
    count_ integer;
BEGIN
    SELECT
        count(users) INTO count_
    FROM ( SELECT DISTINCT
            e.creator_id
        FROM
            edit e
            INNER JOIN person p ON e.creator_id = p.id
        WHERE
            e.published > ('now'::timestamp - i::interval)
            AND p.local = TRUE) AS users;
    RETURN count_;
END;
$$;


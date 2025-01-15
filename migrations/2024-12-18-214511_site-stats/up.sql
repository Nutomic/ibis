CREATE TABLE instance_stats (
    id serial PRIMARY KEY,
    users int NOT NULL DEFAULT 0,
    users_active_month int NOT NULL DEFAULT 0,
    users_active_half_year int NOT NULL DEFAULT 0,
    articles int NOT NULL DEFAULT 0
);

INSERT INTO instance_stats (users, articles)
SELECT
    (
        SELECT
            count(*)
        FROM
            local_user) AS users,
    (
        SELECT
            count(*)
        FROM
            article
        WHERE
            local = TRUE) AS article
FROM
    instance;

CREATE FUNCTION instance_stats_local_user_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        instance_stats
    SET
        users = users + 1;
    RETURN NULL;
END
$$;

CREATE FUNCTION instance_stats_local_user_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        instance_stats sa
    SET
        users = users - 1
    FROM
        instance s
    WHERE
        sa.instance_id = s.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER instance_stats_local_user_insert
    AFTER INSERT ON local_user
    FOR EACH ROW
    EXECUTE PROCEDURE instance_stats_local_user_insert ();

CREATE TRIGGER instance_stats_local_user_delete
    AFTER DELETE ON local_user
    FOR EACH ROW
    EXECUTE PROCEDURE instance_stats_local_user_delete ();

CREATE FUNCTION instance_stats_article_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        instance_stats
    SET
        articles = articles + 1;
    RETURN NULL;
END
$$;

CREATE FUNCTION instance_stats_article_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE
        instance_stats ia
    SET
        articles = articles - 1
    FROM
        instance i
    WHERE
        ia.instance_id = i.id;
    RETURN NULL;
END
$$;

CREATE TRIGGER instance_stats_article_insert
    AFTER INSERT ON article
    FOR EACH ROW
    WHEN (NEW.local = TRUE)
    EXECUTE PROCEDURE instance_stats_article_insert ();

CREATE TRIGGER instance_stats_article_delete
    AFTER DELETE ON article
    FOR EACH ROW
    WHEN (OLD.local = TRUE)
    EXECUTE PROCEDURE instance_stats_article_delete ();

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

UPDATE
    instance_stats
SET
    users_active_month = (
        SELECT
            *
        FROM
            instance_stats_activity ('1 month'));

UPDATE
    instance_stats
SET
    users_active_half_year = (
        SELECT
            *
        FROM
            instance_stats_activity ('6 months'));


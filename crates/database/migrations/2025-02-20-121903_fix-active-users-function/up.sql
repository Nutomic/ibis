
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
            INNER JOIN person p ON c.creator_id = p.id
        WHERE
            c.published > ('now'::timestamp - i::interval)
            AND p.local = TRUE) a;
    RETURN count_;
END;
$$;

-- only allow one row in instance_stats
delete from instance_stats;
alter table instance_stats drop column id;
alter table instance_stats add column id int primary key GENERATED ALWAYS AS (1) STORED UNIQUE;
insert into instance_stats values(0,0,0,0,0);
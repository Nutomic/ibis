DROP TABLE instance_stats;

DROP TRIGGER instance_stats_local_user_insert ON local_user;

DROP TRIGGER instance_stats_local_user_delete ON local_user;

DROP TRIGGER instance_stats_article_insert ON article;

DROP TRIGGER instance_stats_article_delete ON article;

DROP FUNCTION instance_stats_local_user_insert, instance_stats_local_user_delete, instance_stats_article_insert, instance_stats_article_delete, instance_stats_activity;


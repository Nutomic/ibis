UPDATE
  article
SET
  title = REPLACE(title, '_', ' ')
WHERE
  local;
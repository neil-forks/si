SELECT password
FROM users
WHERE users.pk = $1 AND users.visibility_deleted_at IS NULL

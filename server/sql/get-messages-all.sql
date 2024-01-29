SELECT "hash", "sender", "receiver", "data_hash", "timestamp"
FROM "messages"
WHERE
CASE
    WHEN $1=1 THEN "sender"=$2 OR "receiver"=$2
    ELSE "receiver"=$2
END
ORDER BY "timestamp" ASC
OFFSET $3
LIMIT $4;

SELECT "hash", "sender", "receiver", "data_hash", "timestamp"
FROM "messages"
WHERE
CASE
    WHEN $1=1 THEN "sender"=$2 AND "receiver"=$3 OR "sender"=$3 AND "receiver"=$2
    ELSE "sender"=$3 AND "receiver"=$2
END
ORDER BY "timestamp" ASC
OFFSET $4
LIMIT $5;

SELECT "hash", "sender", "receiver", "data_hash", "timestamp", "success"
FROM "messages"
WHERE "sender"=?1 AND "receiver"=?2 OR "sender"=?2 AND "receiver"=?1
ORDER BY "timestamp" ASC;
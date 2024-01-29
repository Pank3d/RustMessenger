DELETE FROM "messages"
WHERE ("sender"=$1 OR "receiver"=$1) AND "hash"=ANY($2)
RETURNING "hash";

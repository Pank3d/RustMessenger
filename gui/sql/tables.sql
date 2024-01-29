CREATE TABLE IF NOT EXISTS "accounts" (
    "sk" VARCHAR(44),
    "pk" VARCHAR(44),
    "name" VARCHAR(128)
);
CREATE TABLE IF NOT EXISTS "contacts" (
    "pk" VARCHAR(44),
    "name" VARCHAR(128)
);
CREATE TABLE IF NOT EXISTS "messages" (
    "hash" VARCHAR(88),
    "sender" VARCHAR(44),
    "receiver" VARCHAR(44),
    "data_hash" VARCHAR(88),
    "timestamp" TIMESTAMP,
    "success" BOOLEAN
);

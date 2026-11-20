ALTER TABLE room_instance ADD COLUMN matchmaking_policy INTEGER NOT NULL DEFAULT 0
    CHECK (matchmaking_policy IN (0, 1, 2));
PRAGMA user_version=8;

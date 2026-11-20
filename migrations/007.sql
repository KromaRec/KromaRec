ALTER TABLE local_account ADD COLUMN access_generation INTEGER NOT NULL DEFAULT 0;
ALTER TABLE local_account ADD COLUMN photon_generation INTEGER NOT NULL DEFAULT 0;
PRAGMA user_version=7;

CREATE TABLE photon_ban (
    player INTEGER PRIMARY KEY REFERENCES local_account(id) ON DELETE CASCADE,
    expires INTEGER CHECK(expires IS NULL OR expires > 0)
);
PRAGMA user_version=6;

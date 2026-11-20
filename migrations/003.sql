ALTER TABLE local_account ADD COLUMN bio TEXT NOT NULL DEFAULT '';
ALTER TABLE local_account ADD COLUMN identity_flags INTEGER NOT NULL DEFAULT 0;
ALTER TABLE local_account ADD COLUMN personal_pronouns INTEGER NOT NULL DEFAULT 0;

CREATE TABLE relationship (
    owner INTEGER NOT NULL,
    peer INTEGER NOT NULL,
    kind INTEGER NOT NULL DEFAULT 0 CHECK(kind BETWEEN 0 AND 3),
    favorited INTEGER NOT NULL DEFAULT 0,
    ignored INTEGER NOT NULL DEFAULT 0,
    muted INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY(owner, peer),
    CHECK(owner <> peer)
) WITHOUT ROWID;

CREATE TABLE social_inbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sender INTEGER NOT NULL,
    recipient INTEGER NOT NULL,
    kind INTEGER NOT NULL,
    sent TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
);
CREATE INDEX social_inbox_recipient ON social_inbox(recipient,id DESC);
CREATE TABLE lobby_presence (account_id INTEGER PRIMARY KEY, login_lock TEXT NOT NULL, expires INTEGER NOT NULL);
CREATE INDEX lobby_expiry ON lobby_presence(expires);
PRAGMA user_version=3;

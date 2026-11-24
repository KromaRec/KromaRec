CREATE TABLE room_comment (
  comment_id INTEGER PRIMARY KEY AUTOINCREMENT,
  room_id INTEGER NOT NULL, subroom_id INTEGER NOT NULL, player_id INTEGER NOT NULL,
  style INTEGER NOT NULL DEFAULT 0, message TEXT NOT NULL DEFAULT '',
  position_x REAL NOT NULL DEFAULT 0, position_y REAL NOT NULL DEFAULT 0,
  position_z REAL NOT NULL DEFAULT 0, created_at TEXT NOT NULL
);
CREATE INDEX room_comment_lookup ON room_comment(room_id,comment_id);
CREATE TABLE club (
  club_id INTEGER PRIMARY KEY, data TEXT NOT NULL CHECK(json_type(data)='object')
);
CREATE TABLE curated_list (
  creator_id INTEGER NOT NULL, entity_type INTEGER NOT NULL, name TEXT NOT NULL COLLATE NOCASE,
  data TEXT NOT NULL CHECK(json_type(data)='object'),
  PRIMARY KEY(creator_id,entity_type,name)
);
CREATE TABLE studio_cloud_build (
  cloud_build_id TEXT PRIMARY KEY, started_at TEXT NOT NULL, completed_at TEXT,
  error TEXT, unity_asset_id TEXT NOT NULL, created_by_account_id INTEGER NOT NULL
);
PRAGMA user_version=9;

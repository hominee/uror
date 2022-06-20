-- Your SQL goes here
-- id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL ,
create table if not exists uris(
	uri TEXT PRIMARY KEY NOT NULL,
	iner TEXT NOT NULL
);

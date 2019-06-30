ALTER TABLE note RENAME TO _note_0_3;

CREATE TABLE IF NOT EXISTS note (
rowid          INTEGER PRIMARY KEY AUTOINCREMENT,
uuid4          TEXT NOT NULL,
title          TEXT NOT NULL,
url            TEXT NOT NULL,
tags           TEXT NOT NULL,
description    TEXT NOT NULL,
comments       TEXT NOT NULL,
annotations    TEXT NOT NULL,
created_at     TEXT NOT NULL,
is_public      BOOLEAN NOT NULL default 0
);

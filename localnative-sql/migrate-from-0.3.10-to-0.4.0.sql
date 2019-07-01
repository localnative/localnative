ALTER TABLE note RENAME TO _note_0_3;

CREATE TABLE IF NOT EXISTS note (
rowid          INTEGER PRIMARY KEY AUTOINCREMENT,
uuid4          TEXT NOT NULL UNIQUE,
title          TEXT NOT NULL,
url            TEXT NOT NULL,
tags           TEXT NOT NULL,
description    TEXT NOT NULL,
comments       TEXT NOT NULL,
annotations    TEXT NOT NULL,
created_at     TEXT NOT NULL,
is_public      BOOLEAN NOT NULL default 0
);

-- dedup

ALTER TABLE note RENAME TO _note_dedup;

insert into note (uuid4, title, url, tags, description, comments, annotations, created_at, is_public)
with a as (
select *, row_number() over (PARTITION by title, url, tags, description, comments, annotations, created_at, is_public order by rowid) as r
from _note_dedup
) select uuid4, title, url, tags, description, comments, annotations, created_at, is_public from a where r = 1
order by rowid;

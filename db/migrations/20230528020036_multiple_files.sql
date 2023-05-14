-- Add migration script here
ALTER TABLE questions ADD COLUMN attachments TEXT NOT NULL DEFAULT '[]';
ALTER TABLE questions DROP COLUMN attachment;
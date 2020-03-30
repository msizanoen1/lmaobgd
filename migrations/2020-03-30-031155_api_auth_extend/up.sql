ALTER TABLE api_keys
    ADD COLUMN write_access BOOLEAN NOT NULL DEFAULT 't',
    ADD COLUMN note TEXT;
ALTER TABLE api_keys
    ALTER COLUMN write_access DROP DEFAULT;

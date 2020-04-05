CREATE EXTENSION pgcrypto;

CREATE TEMPORARY TABLE group_map AS
SELECT
    id,
    gen_random_uuid() AS uuid
FROM groups;
CREATE TEMPORARY TABLE api_map AS
SELECT
    id,
    gen_random_uuid() AS uuid
FROM api_keys;
CREATE INDEX t1 ON api_map(id);
CREATE INDEX t2 ON group_map(id);

CREATE FUNCTION group_uuid(INTEGER) RETURNS UUID AS $$
DECLARE
    result UUID;
BEGIN
    SELECT uuid
    INTO result
    FROM group_map
    WHERE id = $1;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'WTF';
    END IF;
    RETURN result;
END
$$ LANGUAGE plpgsql;
CREATE FUNCTION api_uuid(INTEGER) RETURNS UUID AS $$
DECLARE
    result UUID;
BEGIN
    SELECT uuid
    INTO result
    FROM api_map
    WHERE id = $1;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'WTF';
    END IF;
    RETURN result;
END
$$ LANGUAGE plpgsql;
ALTER TABLE answers
    DROP CONSTRAINT answers_test_fkey;
ALTER TABLE groups
    ALTER COLUMN id DROP DEFAULT;
ALTER TABLE api_keys
    ALTER COLUMN id DROP DEFAULT;
ALTER TABLE answers
    ALTER COLUMN test TYPE UUID USING group_uuid(test);
ALTER TABLE groups
    ALTER COLUMN id TYPE UUID USING group_uuid(id),
    ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE api_keys
    ALTER COLUMN id TYPE UUID USING api_uuid(id),
    ALTER COLUMN id SET DEFAULT gen_random_uuid();
ALTER TABLE answers
    ADD CONSTRAINT answers_test_fkey FOREIGN KEY (test) REFERENCES groups (id);
DROP FUNCTION api_uuid;
DROP FUNCTION group_uuid;
DROP INDEX t1;
DROP INDEX t2;

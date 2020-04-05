
CREATE TEMPORARY SEQUENCE api_seq;
CREATE TEMPORARY SEQUENCE group_seq;
CREATE TEMPORARY TABLE group_map AS
SELECT
    nextval('group_seq') AS id,
    id AS uuid
FROM groups;
CREATE TEMPORARY TABLE api_map AS
SELECT
    nextval('api_seq') AS id,
    id AS uuid
FROM api_keys;
CREATE INDEX t1 ON group_map(uuid);
CREATE INDEX t2 ON api_map(uuid);
CREATE FUNCTION group_iid(UUID) RETURNS INTEGER AS $$
DECLARE
    result INTEGER;
BEGIN
    SELECT id
    INTO result
    FROM group_map
    WHERE uuid=$1;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'WTF';
    END IF;
    RETURN result;
END
$$ LANGUAGE plpgsql;
CREATE FUNCTION api_iid(UUID) RETURNS INTEGER AS $$
DECLARE
    result INTEGER;
BEGIN
    SELECT id
    INTO result
    FROM api_map
    WHERE uuid=$1;
    IF NOT FOUND THEN
        RAISE EXCEPTION 'WTF';
    END IF;
    RETURN result;
END
$$ LANGUAGE plpgsql;
ALTER TABLE answers
    DROP CONSTRAINT answers_test_fkey,
    ALTER COLUMN test TYPE INTEGER USING group_iid(test);
ALTER TABLE groups
    ALTER COLUMN id DROP DEFAULT;
ALTER TABLE groups
    ALTER COLUMN id TYPE INTEGER USING group_iid(id),
    ALTER COLUMN id SET DEFAULT nextval('groups_id_seq');
ALTER TABLE answers
    ADD CONSTRAINT answers_test_fkey FOREIGN KEY (test) REFERENCES groups (id);
ALTER TABLE api_keys
    ALTER COLUMN id DROP DEFAULT;
ALTER TABLE api_keys
    ALTER COLUMN id TYPE INTEGER USING api_iid(id),
    ALTER COLUMN id SET DEFAULT nextval('api_keys_id_seq');
DROP FUNCTION api_iid;
DROP FUNCTION group_iid;
DROP EXTENSION pgcrypto;
DROP INDEX t1;
DROP INDEX t2;
DROP TABLE group_map;
DROP TABLE api_map;

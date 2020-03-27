CREATE TABLE api_keys (
    id SERIAL PRIMARY KEY,
    hash bytea NOT NULL
);
CREATE UNIQUE INDEX api_key_idx ON api_keys (hash);

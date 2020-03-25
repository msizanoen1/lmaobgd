CREATE FUNCTION get_answer_strings(INTEGER[]) RETURNS TEXT[] AS $LMAO$
DECLARE
    id INTEGER;
    idx INTEGER := 1;
    result TEXT[] := ARRAY[]::TEXT[];
    current_text TEXT;
BEGIN
    FOREACH id IN ARRAY $1
    LOOP
        FOR current_text IN SELECT
            answer_string AS current_text
        FROM
            answer_strings
        WHERE
            answer_id = id
        LOOP
            result[idx] := current_text;
        END LOOP;
        IF NOT FOUND THEN
            RAISE EXCEPTION 'No string for answer %1', id;
        END IF;
        idx := idx + 1;
    END LOOP;
    RETURN result;
END
$LMAO$
LANGUAGE plpgsql;

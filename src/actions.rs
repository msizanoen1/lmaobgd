use crate::models::*;
use crate::schema::*;
use blake2::Blake2b;
use diesel::pg::expression::dsl::any;
use diesel::pg::upsert::excluded;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::QueryResult;
use digest::Digest;
use once_cell::sync::Lazy;
use rand::prelude::*;
use std::borrow::Cow;
use std::collections::HashMap;

fn generate_api_key(length: u64) -> String {
    static CHARS: Lazy<Vec<char>> = Lazy::new(|| {
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
            .chars()
            .into_iter()
            .collect()
    });
    let mut result = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..length {
        let idx = rng.gen_range(0, CHARS.len());
        result.push(CHARS[idx]);
    }
    result
}

pub fn get_answer_string(conn: &PgConnection, id: i32) -> QueryResult<String> {
    Ok(answer_strings::table
        .find(id)
        .get_result::<AnswerMap>(conn)?
        .answer_string)
}

pub fn get_question_string(conn: &PgConnection, id: i32) -> QueryResult<String> {
    Ok(question_strings::table
        .find(id)
        .get_result::<QuestionMap>(conn)?
        .question_string)
}

pub fn upload_call(conn: &PgConnection, data: JsApiUpload) -> QueryResult<()> {
    let answer_map = data
        .answer_map
        .iter()
        .map(|tup| NewAnswerMap {
            answer_id: *tup.0,
            answer_string: &tup.1,
        })
        .collect::<Vec<_>>();
    diesel::insert_into(answer_strings::table)
        .values(&answer_map)
        .on_conflict(answer_strings::answer_id)
        .do_update()
        .set(answer_strings::answer_string.eq(excluded(answer_strings::answer_string)))
        .execute(conn)?;
    let question_map = data
        .question_map
        .iter()
        .map(|tup| NewQuestionMap {
            question_id: *tup.0,
            question_string: &tup.1,
        })
        .collect::<Vec<_>>();
    diesel::insert_into(question_strings::table)
        .values(&question_map)
        .on_conflict(question_strings::question_id)
        .do_update()
        .set(question_strings::question_string.eq(excluded(question_strings::question_string)))
        .execute(conn)?;
    let group_text = data.group_text;
    diesel::insert_into(groups::table)
        .values(&NewGroup { text: &group_text })
        .on_conflict_do_nothing()
        .execute(conn)?;
    let answers = data
        .unknown_questions
        .into_iter()
        .map(|(id, guess)| Answer {
            valid_answers: guess.answers.clone(),
            answer_used: guess.answer_used,
            question_id: id,
            reviewed: false,
            test: Cow::from(&group_text),
        })
        .collect::<Vec<_>>();
    diesel::insert_into(answers::table)
        .values(&answers)
        .on_conflict(answers::question_id)
        .do_update()
        .set((
            answers::answer_used.eq(excluded(answers::answer_used)),
            answers::reviewed.eq(false),
            answers::test.eq(&group_text),
            answers::valid_answers.eq(excluded(answers::valid_answers)),
        ))
        .execute(conn)?;
    Ok(())
}

pub fn get_data(conn: &PgConnection) -> QueryResult<HashMap<i32, i32>> {
    Ok(answers::table
        .filter(answers::reviewed)
        .select((answers::question_id, answers::answer_used))
        .load(conn)?
        .into_iter()
        .collect())
}

pub fn set_reviewed(conn: &PgConnection, ids: &[i32]) -> QueryResult<()> {
    diesel::update(answers::table.filter(answers::question_id.eq(any(ids))))
        .set(answers::reviewed.eq(true))
        .execute(conn)?;
    Ok(())
}

pub fn gen_api_key(
    conn: &PgConnection,
    note: Option<&str>,
    write_access: bool,
) -> QueryResult<String> {
    let key = generate_api_key(128);
    let hash = Blake2b::digest(key.as_bytes());
    let new = NewApiKey {
        hash: &hash[..],
        note,
        write_access,
    };
    diesel::insert_into(api_keys::table)
        .values(&new)
        .execute(conn)?;
    Ok(key)
}

pub fn check_api_key(conn: &PgConnection, key: &str) -> QueryResult<Option<(i32, Option<String>)>> {
    let hash = Blake2b::digest(key.as_bytes());
    Ok(api_keys::table
        .filter(api_keys::write_access)
        .filter(api_keys::hash.eq(&hash[..]))
        .select((api_keys::id, api_keys::note))
        .first(conn)
        .optional()?)
}

pub fn check_api_key_r(
    conn: &PgConnection,
    key: &str,
) -> QueryResult<Option<(i32, Option<String>)>> {
    let hash = Blake2b::digest(key.as_bytes());
    Ok(api_keys::table
        .filter(api_keys::hash.eq(&hash[..]))
        .select((api_keys::id, api_keys::note))
        .first(conn)
        .optional()?)
}

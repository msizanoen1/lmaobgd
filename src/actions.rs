use crate::models::*;
use crate::schema::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::QueryResult;
use std::collections::HashMap;

pub fn js_upload_call(conn: &PgConnection, data: JsApiUpload) -> QueryResult<()> {
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
        .on_conflict_do_nothing()
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
        .on_conflict_do_nothing()
        .execute(conn)?;
    let answers = data
        .unknown_questions
        .into_iter()
        .map(|(id, guess)| Answer {
            answer1: guess.answers[0],
            answer2: guess.answers[1],
            answer3: guess.answers[2],
            answer4: guess.answers[3],
            answer_used: guess.answer_used,
            question_id: id,
        })
        .collect::<Vec<_>>();
    diesel::insert_into(answers::table)
        .values(&answers)
        .execute(conn)?;
    Ok(())
}

pub fn js_get_data(conn: &PgConnection) -> QueryResult<HashMap<i32, i32>> {
    Ok(answers::table
        .get_results::<Answer>(conn)?
        .into_iter()
        .map(|ans| (ans.question_id, ans.answer_used))
        .collect())
}

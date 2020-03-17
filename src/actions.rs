use crate::models::*;
use crate::schema::*;
use diesel::pg::upsert::excluded;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::QueryResult;
use std::collections::HashMap;

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
    let group = data.group;
    let group_text = data.group_text;
    diesel::insert_into(groups::table)
        .values(&NewGroup {
            id: group,
            text: &group_text,
        })
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
            reviewed: false,
            group_: Some(group),
        })
        .collect::<Vec<_>>();
    diesel::insert_into(answers::table)
        .values(&answers)
        .on_conflict(answers::question_id)
        .do_update()
        .set((
            answers::answer_used.eq(excluded(answers::answer_used)),
            answers::reviewed.eq(false),
            answers::group_.eq(Some(group)),
        ))
        .execute(conn)?;
    Ok(())
}

pub fn js_get_data(conn: &PgConnection) -> QueryResult<HashMap<i32, i32>> {
    Ok(answers::table
        .filter(answers::reviewed.eq(true))
        .get_results::<Answer>(conn)?
        .into_iter()
        .map(|ans| (ans.question_id, ans.answer_used))
        .collect())
}

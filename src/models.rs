use crate::schema::*;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Insertable, Queryable)]
#[table_name = "answers"]
pub struct Answer {
    pub question_id: i32,
    pub answer1: i32,
    pub answer2: i32,
    pub answer3: i32,
    pub answer4: i32,
    pub answer_used: i32,
    pub reviewed: bool,
    pub group_: Option<i32>,
}

#[derive(Queryable)]
pub struct AnswerMap {
    pub answer_id: i32,
    pub answer_string: String,
}

#[derive(Queryable)]
pub struct QuestionMap {
    pub question_id: i32,
    pub question_string: String,
}

#[derive(Queryable)]
pub struct Group {
    pub id: i32,
    pub text: String,
}

#[derive(Insertable)]
#[table_name = "answer_strings"]
pub struct NewAnswerMap<'a> {
    pub answer_id: i32,
    pub answer_string: &'a str,
}

#[derive(Insertable)]
#[table_name = "question_strings"]
pub struct NewQuestionMap<'a> {
    pub question_id: i32,
    pub question_string: &'a str,
}

#[derive(Insertable)]
#[table_name = "groups"]
pub struct NewGroup<'a> {
    pub id: i32,
    pub text: &'a str,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsApiUpload {
    pub group: i32,
    pub group_text: String,
    pub answer_map: HashMap<i32, String>,
    pub question_map: HashMap<i32, String>,
    pub unknown_questions: HashMap<i32, UnknownQuestion>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownQuestion {
    pub answers: [i32; 4],
    pub answer_used: i32,
}

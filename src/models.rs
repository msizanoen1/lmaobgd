use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Insertable, Queryable, Clone)]
#[table_name = "answers"]
pub struct Answer {
    pub question_id: i32,
    pub answer_used: i32,
    pub reviewed: bool,
    pub test: i32,
    pub valid_answers: Vec<i32>,
}

#[derive(Queryable, Clone)]
pub struct AnswerMap {
    pub answer_id: i32,
    pub answer_string: String,
}

#[derive(Queryable, Clone)]
pub struct QuestionMap {
    pub question_id: i32,
    pub question_string: String,
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

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsApiUpload {
    pub group: Option<i32>,
    pub group_text: String,
    pub answer_map: HashMap<i32, String>,
    pub question_map: HashMap<i32, String>,
    pub unknown_questions: HashMap<i32, UnknownQuestion>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnknownQuestion {
    pub answers: Vec<i32>,
    pub answer_used: i32,
}

#[derive(Queryable, Clone)]
pub struct ApiKey {
    pub id: i32,
    pub hash: Vec<u8>,
    pub write_access: bool,
    pub note: Option<String>,
}

#[derive(Insertable, Clone)]
#[table_name = "api_keys"]
pub struct NewApiKey<'a> {
    pub hash: &'a [u8],
    pub write_access: bool,
    pub note: Option<&'a str>,
}

#[derive(Queryable, Eq, PartialEq, Ord, PartialOrd)]
pub struct Group {
    pub id: i32,
    pub text: String,
}

#[derive(Insertable)]
#[table_name = "groups"]
pub struct Groups<'a> {
    pub text: &'a str,
}

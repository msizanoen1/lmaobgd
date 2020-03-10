table! {
    answers (question_id) {
        question_id -> Int4,
        answer1 -> Int4,
        answer2 -> Int4,
        answer3 -> Int4,
        answer4 -> Int4,
        answer_used -> Int4,
        reviewed -> Bool,
    }
}

table! {
    answer_strings (answer_id) {
        answer_id -> Int4,
        answer_string -> Text,
    }
}

table! {
    question_strings (question_id) {
        question_id -> Int4,
        question_string -> Text,
    }
}

joinable!(answers -> question_strings (question_id));

allow_tables_to_appear_in_same_query!(
    answers,
    answer_strings,
    question_strings,
);

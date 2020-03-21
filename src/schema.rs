table! {
    answer_strings (answer_id) {
        answer_id -> Int4,
        answer_string -> Text,
    }
}

table! {
    answers (question_id) {
        question_id -> Int4,
        answer_used -> Int4,
        reviewed -> Bool,
        test_id -> Int4,
        valid_answers -> Array<Int4>,
    }
}

table! {
    groups (id) {
        id -> Int4,
        text -> Text,
    }
}

table! {
    question_strings (question_id) {
        question_id -> Int4,
        question_string -> Text,
    }
}

joinable!(answers -> answer_strings (answer_used));
joinable!(answers -> groups (test_id));
joinable!(answers -> question_strings (question_id));

allow_tables_to_appear_in_same_query!(
    answer_strings,
    answers,
    groups,
    question_strings,
);

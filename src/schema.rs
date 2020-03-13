table! {
    answer_strings (answer_id) {
        answer_id -> Int4,
        answer_string -> Text,
    }
}

table! {
    answers (question_id) {
        question_id -> Int4,
        answer1 -> Int4,
        answer2 -> Int4,
        answer3 -> Int4,
        answer4 -> Int4,
        answer_used -> Int4,
        reviewed -> Bool,
        group_ -> Nullable<Int4>,
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

joinable!(answers -> groups (group_));
joinable!(answers -> question_strings (question_id));

allow_tables_to_appear_in_same_query!(
    answer_strings,
    answers,
    groups,
    question_strings,
);

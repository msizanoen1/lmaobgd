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
        test -> Uuid,
        valid_answers -> Array<Int4>,
    }
}

table! {
    api_keys (id) {
        id -> Uuid,
        hash -> Bytea,
        write_access -> Bool,
        note -> Nullable<Text>,
    }
}

table! {
    groups (id) {
        id -> Uuid,
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
joinable!(answers -> groups (test));
joinable!(answers -> question_strings (question_id));

allow_tables_to_appear_in_same_query!(
    answer_strings,
    answers,
    api_keys,
    groups,
    question_strings,
);

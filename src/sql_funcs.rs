use diesel::sql_types::*;

sql_function! {
    fn get_answer_strings(input: Array<Int4>) -> Array<Text>;
}

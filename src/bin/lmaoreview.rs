use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use exitfailure::ExitFailure;
use failure::ResultExt;
use lmaobgd::actions;
use lmaobgd::models::*;
use lmaobgd::schema::*;
use std::env;
use std::io::stdin;

fn main() -> Result<(), ExitFailure> {
    let _ = dotenv();
    let url = env::var("DATABASE_URL").context("unable to get DATABASE_URL")?;
    let db = PgConnection::establish(&url).context("unable to connect database")?;
    let unreviewed = answers::table
        .filter(answers::reviewed.eq(false))
        .get_results::<Answer>(&db)
        .context("unable to get unreviewed data")?;
    for answer in unreviewed {
        let question_id = answer.question_id;
        let question = actions::get_question_string(&db, question_id)?;
        println!("Question {} ({}):", question_id, question);
        println!("Possible answers:");
        println!(
            "{} ({})",
            answer.answer1,
            actions::get_answer_string(&db, answer.answer1)?
        );
        println!(
            "{} ({})",
            answer.answer2,
            actions::get_answer_string(&db, answer.answer2)?
        );
        println!(
            "{} ({})",
            answer.answer3,
            actions::get_answer_string(&db, answer.answer3)?
        );
        println!(
            "{} ({})",
            answer.answer4,
            actions::get_answer_string(&db, answer.answer4)?
        );
        println!(
            "Answer used: {} ({})",
            answer.answer_used,
            actions::get_answer_string(&db, answer.answer_used)?
        );
        loop {
            print!(
                r#"Select action:
0. This is correct
1. Set correct answer
2. Delete
"#
            );
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            let input = input.trim().parse::<u8>()?;
            if input == 0 {
                diesel::update(answers::table.find(answer.question_id))
                    .set(answers::reviewed.eq(true))
                    .execute(&db)?;
            } else if input == 1 {
                println!("Enter DB ID:");
                let mut input = String::new();
                stdin().read_line(&mut input)?;
                let input = input.trim().parse::<i32>()?;
                diesel::update(answers::table.find(answer.question_id))
                    .set((answers::answer_used.eq(input), answers::reviewed.eq(true)))
                    .execute(&db)?;
                println!(
                    "Updated question {} ({}) to {} ({})",
                    answer.question_id,
                    actions::get_question_string(&db, answer.question_id)?,
                    input,
                    actions::get_answer_string(&db, input)?
                );
            } else if input == 2 {
                diesel::delete(answers::table.find(answer.question_id)).execute(&db)?;
            } else {
                continue;
            }
            break;
        }
    }
    Ok(())
}

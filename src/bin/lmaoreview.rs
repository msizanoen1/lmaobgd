use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::QueryResult;
use dotenv::dotenv;
use exitfailure::ExitFailure;
use failure::ResultExt;
use lmaobgd::actions;
use lmaobgd::models::*;
use lmaobgd::schema::*;
use std::env;
use std::io::stdin;

fn process_question(q: &str) -> String {
    let (mut vec, n) = q.lines().map(|s| s.trim()).filter(|s| *s != "").fold(
        (Vec::new(), 0),
        |(mut acc, i), e| {
            if e.starts_with("A:")
                || e.starts_with("B:")
                || e.starts_with("C:")
                || e.starts_with("D:")
            {
                (acc, i + 1)
            } else {
                acc.push(e);
                (acc, i)
            }
        },
    );
    vec.truncate(vec.len() - n);
    let mut iter = vec.into_iter();
    format!(
        "{}. {}",
        iter.next().unwrap_or(""),
        iter.collect::<Vec<_>>().join("\n")
    )
}

fn get_question_string(db: &PgConnection, id: i32) -> QueryResult<String> {
    Ok(process_question(&actions::get_question_string(db, id)?))
}

fn get_answer_string(db: &PgConnection, id: i32) -> QueryResult<String> {
    let answer = actions::get_answer_string(db, id)?;
    let mut iter = answer.lines().map(|s| s.trim()).filter(|s| *s != "");
    Ok(format!(
        "{} {}",
        iter.next().unwrap_or(""),
        iter.collect::<Vec<_>>().join("\n")
    ))
}

fn main() -> Result<(), ExitFailure> {
    let _ = dotenv();
    let url = env::var("DATABASE_URL").context("unable to get DATABASE_URL")?;
    let db = PgConnection::establish(&url).context("unable to connect database")?;
    let groups = groups::table
        .get_results::<Group>(&db)
        .context("unable to get tests")?;
    println!("Tests available:");
    for group in groups {
        println!("{} ({})", group.id, group.text);
    }
    println!("Select test DB ID:");
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let id = input.parse::<i32>()?;
    let unreviewed = answers::table
        .filter(answers::reviewed.eq(false))
        .filter(answers::group_.eq(id))
        .get_results::<Answer>(&db)
        .context("unable to get unreviewed data")?;
    for answer in unreviewed {
        let question_id = answer.question_id;
        let question = get_question_string(&db, question_id)?;
        println!("Question {} ({}):", question_id, question);
        println!("Possible answers:");
        println!(
            "{} ({})",
            answer.answer1,
            get_answer_string(&db, answer.answer1)?
        );
        println!(
            "{} ({})",
            answer.answer2,
            get_answer_string(&db, answer.answer2)?
        );
        println!(
            "{} ({})",
            answer.answer3,
            get_answer_string(&db, answer.answer3)?
        );
        println!(
            "{} ({})",
            answer.answer4,
            get_answer_string(&db, answer.answer4)?
        );
        println!(
            "Answer used: {} ({})",
            answer.answer_used,
            get_answer_string(&db, answer.answer_used)?
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
                    get_question_string(&db, answer.question_id)?,
                    input,
                    get_answer_string(&db, input)?
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

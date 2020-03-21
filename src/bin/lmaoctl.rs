use diesel::dsl::exists;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::QueryResult;
use dotenv::dotenv;
use exitfailure::ExitFailure;
use failure::ResultExt;
use lmaobgd::actions;
use lmaobgd::models::*;
use lmaobgd::schema::*;
use std::collections::HashSet;
use std::io::{stdin, Write};
use structopt::StructOpt;

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

fn process_question2(q: &str) -> String {
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
    let iter = vec.into_iter().skip(1);
    format!("{}", iter.collect::<Vec<_>>().join("\n"))
}

fn get_question_string(db: &PgConnection, id: i32) -> QueryResult<String> {
    Ok(process_question(&actions::get_question_string(db, id)?))
}

fn get_question_string2(db: &PgConnection, id: i32) -> QueryResult<String> {
    Ok(process_question2(&actions::get_question_string(db, id)?))
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

fn get_answer_string2(db: &PgConnection, id: i32) -> QueryResult<String> {
    let answer = actions::get_answer_string(db, id)?;
    let iter = answer
        .lines()
        .map(|s| s.trim())
        .filter(|s| *s != "")
        .skip(1);
    Ok(format!("{}", iter.collect::<Vec<_>>().join("\n")))
}

#[derive(StructOpt)]
enum Command {
    Review,
    ViewData,
    DeleteQuestion { id: i32 },
    Dump,
    Collapse,
}

#[derive(StructOpt)]
struct Args {
    #[structopt(short, long, env = "DATABASE_URL")]
    database_url: String,
    #[structopt(subcommand)]
    command: Command,
}

fn main() -> Result<(), ExitFailure> {
    let _ = dotenv();
    let args = Args::from_args();
    let url = args.database_url;
    let db = PgConnection::establish(&url).context("unable to connect database")?;
    match args.command {
        Command::Review => review(db)?,
        Command::ViewData => view(db)?,
        Command::DeleteQuestion { id } => del_question(db, id)?,
        Command::Dump => dump(db)?,
        Command::Collapse => collapse(db)?,
    }
    Ok(())
}

fn collapse(db: PgConnection) -> Result<(), failure::Error> {
    let group_texts = groups::table
        .select(groups::text)
        .load::<String>(&db)?
        .into_iter()
        .collect::<HashSet<_>>();
    for text in group_texts {
        let group_ids = groups::table
            .filter(groups::text.eq(&text))
            .select(groups::id)
            .load::<i32>(&db)?;
        if group_ids.len() > 1 {
            // There is many group of same name
            let base_id = group_ids[0];
            let other_ids = &group_ids[1..];
            diesel::update(answers::table.filter(answers::test_id.eq_any(other_ids)))
                .set(answers::test_id.eq(base_id))
                .execute(&db)?;
        }
    }
    Ok(())
}

fn dump(db: PgConnection) -> Result<(), failure::Error> {
    let groups = group_rev(&db)?;
    for group in groups {
        let file_name = format!("{}.txt", group.text);
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_name)?;
        let answers = answers::table
            .filter(
                answers::test_id
                    .eq(group.id)
                    .and(answers::reviewed.eq(true)),
            )
            .load::<Answer>(&db)?;
        for answer in answers {
            let atext = get_answer_string2(&db, answer.answer_used)?;
            let qtext = get_question_string2(&db, answer.question_id)?;
            writeln!(file, "{}:::{}", qtext, atext)?;
        }
    }
    Ok(())
}

fn del_question(db: PgConnection, id: i32) -> Result<(), failure::Error> {
    diesel::delete(answers::table.find(id)).execute(&db)?;
    Ok(())
}

fn groups(db: &PgConnection) -> Result<Vec<Group>, failure::Error> {
    Ok(groups::table
        .filter(exists(
            answers::table.filter(groups::id.eq(answers::test_id)),
        ))
        .load(db)
        .context("unable to get groups")?)
}

fn group_unrev(db: &PgConnection) -> Result<Vec<Group>, failure::Error> {
    Ok(groups::table
        .filter(exists(
            answers::table.filter(
                groups::id
                    .eq(answers::test_id)
                    .and(answers::reviewed.eq(false)),
            ),
        ))
        .load(db)
        .context("unable to get groups")?)
}

fn group_rev(db: &PgConnection) -> Result<Vec<Group>, failure::Error> {
    Ok(groups::table
        .filter(exists(
            answers::table.filter(
                groups::id
                    .eq(answers::test_id)
                    .and(answers::reviewed.eq(true)),
            ),
        ))
        .load(db)
        .context("unable to get groups")?)
}

fn view(db: PgConnection) -> Result<(), failure::Error> {
    let groups = groups(&db)?;
    println!("Tests available:");
    for group in groups {
        println!("{} ({})", group.id, group.text);
    }
    println!("Select test DB ID:");
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let id: i32 = input.trim().parse()?;
    let answers = answers::table
        .filter(answers::test_id.eq(id))
        .load::<Answer>(&db)
        .context("unable to get answers")?;
    println!("Questions:");
    for answer in answers {
        let question_id = answer.question_id;
        let reviewed = if answer.reviewed {
            "reviewed"
        } else {
            "unreviewed"
        };
        let text = get_question_string(&db, question_id)?;
        println!("{} ({}) ({})", question_id, reviewed, text);
    }
    println!("Enter question ID:");
    input.clear();
    stdin().read_line(&mut input)?;
    let id: i32 = input.trim().parse()?;
    let question = answers::table.find(id).get_result::<Answer>(&db)?;
    println!("Question {}: {}", id, get_question_string(&db, id)?);
    println!("Possible answers:");
    for answer in question.valid_answers {
        println!("{} ({})", answer, get_answer_string(&db, answer)?);
    }
    println!(
        "Answer used: {} ({})",
        question.answer_used,
        get_answer_string(&db, question.answer_used)?
    );
    Ok(())
}

fn review(db: PgConnection) -> Result<(), failure::Error> {
    let groups = group_unrev(&db)?;
    println!("Tests available:");
    for group in groups {
        println!("{} ({})", group.id, group.text);
    }
    println!("Select test DB ID:");
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let id = input.trim().parse::<i32>()?;
    let unreviewed = answers::table
        .filter(answers::reviewed.eq(false))
        .filter(answers::test_id.eq(id))
        .load::<Answer>(&db)
        .context("unable to get unreviewed data")?;
    for answer in unreviewed {
        let question_id = answer.question_id;
        let question = get_question_string(&db, question_id)?;
        println!("Question {} ({}):", question_id, question);
        println!("Possible answers:");
        for answer in answer.valid_answers {
            println!("{} ({})", answer, get_answer_string(&db, answer)?);
        }
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
            print!("{}[2J", 27 as char);
            break;
        }
    }
    Ok(())
}

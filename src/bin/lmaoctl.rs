use diesel::dsl::exists;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use exitfailure::ExitFailure;
use failure::ResultExt;
use lmaobgd::models::*;
use lmaobgd::schema::*;
use lmaobgd::sql_funcs::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
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

fn process_answer(s: &str) -> String {
    let mut iter = s.lines().map(|s| s.trim()).filter(|s| *s != "");
    format!(
        "{} {}",
        iter.next().unwrap_or(""),
        iter.collect::<Vec<_>>().join("\n")
    )
}

fn process_answer2(s: &str) -> String {
    let iter = s.lines().map(|s| s.trim()).filter(|s| *s != "").skip(1);
    format!("{}", iter.collect::<Vec<_>>().join("\n"))
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
    let groups =
        groups::table
            .load::<Group>(&db)?
            .into_iter()
            .fold(HashMap::new(), |mut acc, group| {
                acc.entry(group.text)
                    .or_insert_with(|| Vec::new())
                    .push(group.id);
                acc
            });
    for group_ids in groups.values() {
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
    let group_texts = groups::table
        .select(groups::text)
        .load::<String>(&db)?
        .into_iter()
        .collect::<HashSet<_>>();
    for group in group_texts {
        let file_name = format!("{}.txt", group);
        let mut file = File::create(&file_name)?;
        let answers = answers::table
            .filter(exists(
                groups::table
                    .filter(groups::id.eq(answers::test_id))
                    .filter(groups::text.eq(&group)),
            ))
            .filter(answers::reviewed.eq(true))
            .inner_join(question_strings::table)
            .inner_join(answer_strings::table)
            .select((
                question_strings::question_string,
                answer_strings::answer_string,
            ))
            .load::<(String, String)>(&db)?;
        for (qtext, atext) in answers {
            let atext = process_answer2(&atext);
            let qtext = process_question2(&qtext);
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
            answers::table
                .filter(groups::id.eq(answers::test_id))
                .filter(answers::reviewed.eq(false)),
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
        .inner_join(question_strings::table)
        .inner_join(answer_strings::table)
        .select((
            answers::all_columns,
            question_strings::all_columns,
            answer_strings::all_columns,
            get_answer_strings(answers::valid_answers),
        ))
        .load::<(Answer, QuestionMap, AnswerMap, Vec<String>)>(&db)
        .context("unable to get answers")?;
    println!("Questions:");
    let mut question_cache_text = HashMap::new();
    let mut question_cache = HashMap::new();
    let mut answer_cache = HashMap::new();
    for (answer, question, used, valid_text) in answers {
        let question_id = answer.question_id;
        let reviewed = if answer.reviewed {
            "reviewed"
        } else {
            "unreviewed"
        };
        let text = process_question(&question.question_string);
        println!("{} ({}) ({})", question_id, reviewed, text);
        question_cache.insert(question_id, answer.clone());
        question_cache_text.insert(question_id, text);
        answer_cache.insert(used.answer_id, process_answer(&used.answer_string));
        for (idx, id) in answer.valid_answers.iter().copied().enumerate() {
            answer_cache.insert(id, process_answer(&valid_text[idx]));
        }
    }
    println!("Enter question ID:");
    input.clear();
    stdin().read_line(&mut input)?;
    let id: i32 = input.trim().parse()?;
    let question = question_cache
        .get(&id)
        .ok_or_else(|| failure::format_err!("Question not found"))?;
    println!("Question {}: {}", id, question_cache_text.get(&id).unwrap());
    println!("Possible answers:");
    for answer in &question.valid_answers {
        println!("{} ({})", answer, answer_cache.get(answer).unwrap());
    }
    println!(
        "Answer used: {} ({})",
        question.answer_used,
        answer_cache.get(&question.answer_used).unwrap()
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
        .inner_join(question_strings::table)
        .inner_join(answer_strings::table)
        .select((
            answers::all_columns,
            question_strings::all_columns,
            answer_strings::all_columns,
            get_answer_strings(answers::valid_answers),
        ))
        .load::<(Answer, QuestionMap, AnswerMap, Vec<String>)>(&db)
        .context("unable to get unreviewed data")?;
    let mut answer_text_cache = HashMap::new();
    for (answer, question, used, all) in unreviewed {
        let question_text = process_question(&question.question_string);
        println!("Question {} ({}):", question.question_id, question_text);
        println!("Possible answers:");
        for (idx, id) in answer.valid_answers.iter().copied().enumerate() {
            answer_text_cache.insert(answer.valid_answers[idx], process_answer(&all[idx]));
            println!(
                "{} ({})",
                answer.valid_answers[idx],
                answer_text_cache.get(&id).unwrap()
            );
        }
        println!(
            "Answer used: {} ({})",
            used.answer_id,
            process_answer(&used.answer_string)
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
                    question_text,
                    input,
                    answer_text_cache
                        .get(&input)
                        .map(|x| &x[..])
                        .unwrap_or("INVALID ANSWER ID")
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

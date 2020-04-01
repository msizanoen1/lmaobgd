use diesel::dsl::{exists, not};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use exitfailure::ExitFailure;
use failure::ResultExt;
use lmaobgd::actions::gen_api_key;
use lmaobgd::display::*;
use lmaobgd::models::*;
use lmaobgd::schema::*;
use lmaobgd::sql_funcs::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, Write};
use structopt::StructOpt;

#[derive(StructOpt)]
enum Command {
    /// Open review interface
    Review,
    /// Inspect data
    ViewData,
    /// Delete a question by id
    DeleteQuestion {
        /// Id of the question
        id: i32,
    },
    /// Dump answer text to current directory
    Dump,
    /// Generate a new api key
    ApiKey {
        /// Allow write access
        #[structopt(short, long)]
        write: bool,
        /// Note for key
        #[structopt(short, long)]
        note: Option<String>,
    },
    /// List api keys by id and note
    LsApi,
    /// Remove an api key by id
    RmApi {
        /// API key id to remove
        id: i32,
    },
}

/// Command line interface for LmaoBGD administration.
#[derive(StructOpt)]
struct Args {
    /// URL for database connection
    #[structopt(short, long, env = "DATABASE_URL", hide_env_values = true)]
    database_url: String,
    /// Command to execute
    #[structopt(subcommand)]
    command: Command,
}

fn new_api_key(db: PgConnection, write: bool, note: Option<String>) -> Result<(), failure::Error> {
    let key = gen_api_key(&db, note.as_deref(), write)?;
    println!("{}", key);
    Ok(())
}

fn ls_api(db: PgConnection) -> Result<(), failure::Error> {
    let apis = api_keys::table.load::<ApiKey>(&db)?;
    for key in apis {
        print!("id={} write_access={}", key.id, key.write_access);
        if let Some(note) = key.note {
            print!(" (note: {})", note);
        }
        println!();
    }
    Ok(())
}

fn rm_api(db: PgConnection, id: i32) -> Result<(), failure::Error> {
    diesel::delete(api_keys::table.find(id)).execute(&db)?;
    Ok(())
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
        Command::ApiKey { write, note } => new_api_key(db, write, note)?,
        Command::LsApi => ls_api(db)?,
        Command::RmApi { id } => rm_api(db, id)?,
    }
    Ok(())
}

fn dump(db: PgConnection) -> Result<(), failure::Error> {
    let answers: HashMap<String, HashMap<String, String>> = answers::table
        .filter(answers::reviewed)
        .inner_join(groups::table)
        .inner_join(question_strings::table)
        .inner_join(answer_strings::table)
        .select((
            groups::text,
            question_strings::question_string,
            answer_strings::answer_string,
        ))
        .load::<(String, String, String)>(&db)?
        .into_iter()
        .fold(HashMap::new(), |mut groups, (test, question, answer)| {
            groups
                .entry(test)
                .or_insert_with(|| HashMap::new())
                .insert(question, answer);
            groups
        });
    for (text, group) in answers {
        let file_name = format!("{}.txt", text);
        let mut file = File::create(&file_name)?;
        for (qtext, atext) in group {
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
            answers::table.filter(groups::text.eq(answers::test)),
        ))
        .load(db)
        .context("unable to get groups")?)
}

fn group_unrev(db: &PgConnection) -> Result<Vec<Group>, failure::Error> {
    Ok(groups::table
        .filter(exists(
            answers::table
                .filter(groups::text.eq(answers::test))
                .filter(not(answers::reviewed)),
        ))
        .load(db)
        .context("unable to get groups")?)
}

fn view(db: PgConnection) -> Result<(), failure::Error> {
    let mut groups = groups(&db)?;
    groups.sort();
    println!("Tests available:");
    for (idx, group) in groups.iter().enumerate() {
        println!("{} ({})", idx, group.text);
    }
    println!("Select test DB ID:");
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let id: usize = input.trim().parse()?;
    let text = &groups
        .get(id)
        .ok_or_else(|| failure::format_err!("Invalid group"))?
        .text;
    let answers = answers::table
        .filter(answers::test.eq(text))
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
    let mut groups = group_unrev(&db)?;
    groups.sort();
    println!("Tests available:");
    for (idx, group) in groups.iter().enumerate() {
        println!("{} ({})", idx, group.text);
    }
    println!("Select test DB ID:");
    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let id = input.trim().parse::<usize>()?;
    let text = &groups
        .get(id)
        .ok_or_else(|| failure::format_err!("Invalid group"))?
        .text;
    let unreviewed = answers::table
        .filter(not(answers::reviewed))
        .filter(answers::test.eq(text))
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

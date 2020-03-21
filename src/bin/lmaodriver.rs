use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use failure::Error;
use lmaobgd::actions;
use lmaobgd::models::*;
use lmaobgd::schema::*;
use rand::prelude::*;
use std::collections::HashMap;
use std::iter::once;
use std::mem::take;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use structopt::StructOpt;
use thirtyfour::common::scriptargs::ScriptArgs;
use thirtyfour::{By, Capabilities, DesiredCapabilities, Keys, WebDriver, WebDriverCommands};
use tokio::io::{copy, stderr, BufReader};
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::process::{Child, Command};
use tokio::signal::ctrl_c;
use tokio::stream::StreamExt;
use tokio::task::spawn_blocking;
use tokio::time::delay_for;

fn wd_error(e: thirtyfour::error::WebDriverError) -> Error {
    failure::format_err!("WebDriver error: {:?}", e)
}

async fn new_firefox<T>(
    command: Option<T>,
    headless: bool,
    verbose: bool,
) -> Result<(WebDriver, Child), Error>
where
    T: Into<String>,
{
    let command = command
        .map(|x| x.into())
        .unwrap_or_else(|| String::from("geckodriver"));
    let sa = std::net::SocketAddr::from(([0, 0, 0, 0], 0));
    let port = TcpListener::bind(sa).await?.local_addr()?.port();
    let url = format!("http://127.0.0.1:{}", port);
    let mut child = Command::new(command)
        .arg("-v")
        .arg("-p")
        .arg(&port.to_string())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut child_stderr = BufReader::new(child.stdout.take().unwrap());
    let mut lines = (&mut child_stderr).lines();
    loop {
        tokio::select! {
            line = lines.next() => {
                if let Some(line) = line {
                    let line = line?;
                    if line.contains("Listening") && line.contains(&port.to_string()) {
                        if verbose {
                            tokio::spawn(async move {
                                let _ = copy(&mut child_stderr, &mut stderr()).await;
                            });
                        }
                        let mut caps = DesiredCapabilities::firefox();
                        let inner = caps.get_mut().as_object_mut().unwrap();
                        inner.remove("platform");
                        inner.remove("version");
                        if headless {
                            caps.add_firefox_option("args", ["-headless"]).map_err(wd_error)?;
                        }
                        let wd = WebDriver::new(&url, &caps).await.map_err(wd_error)?;
                        return Ok((wd, child));
                    }
                }
            }
            _ = &mut child => {
                failure::bail!("Child errored");
            }
        }
    }
}

fn num_list(data: &str) -> Vec<usize> {
    let (list, st) = data
        .chars()
        .fold((Vec::new(), String::new()), |(mut list, mut st), chr| {
            if chr.is_digit(10) {
                st.push(chr);
            } else {
                if !st.is_empty() {
                    list.push(take(&mut st));
                }
            }
            (list, st)
        });
    list.into_iter()
        .chain(once(st))
        .filter_map(|x| x.parse().ok())
        .collect()
}

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

/// LmaoBGD WebDriver
#[derive(StructOpt)]
struct Args {
    /// URL for PostgreSQL database
    #[structopt(short, long, env = "DATABASE_URL")]
    database_url: String,
    /// Dont't auto close after 30s
    #[structopt(short, long)]
    no_autoclose: bool,
    /// Do not auto submit
    #[structopt(short = "s", long)]
    no_submit: bool,
    /// Headless mode
    #[structopt(short, long)]
    headless: bool,
    /// Fetch question text even when in database
    #[structopt(short, long)]
    force_fetch: bool,
    /// Auto review using results page.
    #[structopt(short, long)]
    autoreview: bool,
    /// Repeat auto review until all correct answer.
    #[structopt(short, long)]
    crack: bool,
    /// Geckodriver command to use
    #[structopt(short, long)]
    geckodriver: Option<String>,
    /// Verbose mode
    #[structopt(short, long)]
    verbose: bool,
    /// Account ID
    id: String,
    /// URL to navigate
    test_url: String,
}

#[tokio::main]
async fn main() -> Result<(), exitfailure::ExitFailure> {
    let _ = dotenv();
    let args = Args::from_args();
    let url = args.database_url.clone();
    let db = Arc::new(Mutex::new(
        spawn_blocking(move || PgConnection::establish(&url)).await??,
    ));
    let (wd, mut child) =
        new_firefox(args.geckodriver.as_ref(), args.headless, args.verbose).await?;

    let main = async {
        wd.get("http://study.hanoi.edu.vn/dang-nhap?returnUrl=/")
            .await
            .map_err(wd_error)?;
        let username = wd
            .find_element(By::Id("UserName"))
            .await
            .map_err(wd_error)?;
        let password = wd
            .find_element(By::Id("Password"))
            .await
            .map_err(wd_error)?;
        username.send_keys(&args.id).await.map_err(wd_error)?;
        password.send_keys(&args.id).await.map_err(wd_error)?;
        let button = wd
            .find_element(By::Id("AjaxLogin"))
            .await
            .map_err(wd_error)?;
        button.click().await.map_err(wd_error)?;

        while run(&wd, &args, Arc::clone(&db)).await? {
            // repeat
        }
        Ok::<_, failure::Error>(())
    };
    tokio::select! {
        _ = ctrl_c() => {
            println!("Shutting down on CTRL-C");
        }
        ret = main => {
            if let Err(e) = ret {
                println!("Error: {}", e);
            }
        }
    };

    if !args.no_autoclose {
        wd.close().await.map_err(wd_error)?;
    }
    child.kill()?;

    Ok(())
}

async fn run(
    wd: &WebDriver,
    args: &Args,
    db: Arc<Mutex<PgConnection>>,
) -> Result<bool, failure::Error> {
    let test_url = &args.test_url;
    let db2 = Arc::clone(&db);
    let data = spawn_blocking(move || actions::js_get_data(&db2.lock().unwrap())).await??;
    wd.get(test_url).await.map_err(wd_error)?;
    let start = wd
        .find_element(By::Id("#start-test"))
        .await
        .map_err(wd_error)?;
    start.click().await.map_err(wd_error)?;
    let title_elem = wd
        .find_element(By::Css("body .row .col-12 h1"))
        .await
        .map_err(wd_error)?;
    let id_elem = wd
        .find_element(By::Css("body .row .row .col-12 div"))
        .await
        .map_err(wd_error)?;
    let title = title_elem.text().await.map_err(wd_error)?;
    let id_str = id_elem.text().await.map_err(wd_error)?;
    let id = id_str
        .rsplit(':')
        .nth(0)
        .map(|x| x.trim().parse::<i32>())
        .transpose()?
        .unwrap_or(0);
    println!("Test name: {}", title);
    println!("Test ID: {}", id);
    let questions = wd
        .find_elements(By::Css(".question-box"))
        .await
        .map_err(wd_error)?;
    let mut question_maps = HashMap::new();
    let mut answer_of_questions = HashMap::new();
    let mut answer_maps = HashMap::new();
    let mut unknowns = HashMap::new();
    let mut question_ids = Vec::new();
    for question in questions {
        let q_id = question
            .get_attribute("data-id")
            .await
            .map_err(wd_error)?
            .parse::<i32>()?;
        question_ids.push(q_id);
        let cur_answer = data.get(&q_id).copied();
        if args.force_fetch || cur_answer.is_none() {
            let q_text = question.text().await.map_err(wd_error)?;
            println!("Question {}: {}", q_id, process_question(&q_text));
            question_maps.insert(q_id, q_text);
        }
        let inputs = question
            .find_elements(By::Css(r#"input[type="radio"]"#))
            .await
            .map_err(wd_error)?;
        let mut answers = Vec::new();
        let mut input_elems = Vec::new();
        let mut answered = false;
        for input in inputs {
            input_elems.push(input.clone());
            let a_id = input
                .get_property("value")
                .await
                .map_err(wd_error)?
                .parse::<i32>()?;
            if cur_answer.is_none() || args.force_fetch {
                let mut sa = ScriptArgs::new();
                sa.push(input.clone()).map_err(wd_error)?;
                let a_text = wd
                    .execute_script_with_args(
                        "return arguments[0].parentNode.parentNode.innerText;",
                        &sa,
                    )
                    .await
                    .map_err(wd_error)?
                    .convert()
                    .map_err(wd_error)?;
                answer_maps.insert(a_id, a_text);
            }
            answers.push(a_id);
            if cur_answer == Some(a_id) {
                input.send_keys(Keys::Space).await.map_err(wd_error)?;
                answered = true;
            }
        }
        if !answered {
            let idx = rand::thread_rng().gen_range(0, answers.len());
            unknowns.insert(q_id, answers[idx]);
            input_elems[idx]
                .send_keys(Keys::Space)
                .await
                .map_err(wd_error)?;
        }

        answer_of_questions.insert(q_id, answers);
    }
    let mut correct: Option<Vec<i32>> = None;
    let mut has_incorrect = false;
    if !args.no_submit {
        wd.execute_script(r#"SendUserTestResultToServer("Đang nộp bài, vui lòng đợi và không thực hiện thêm bất cứ thao tác nào!", 2);"#).await.map_err(wd_error)?;
        println!("Waiting for result page...");
        delay_for(Duration::from_secs(15)).await;
        if args.autoreview {
            let correct_e = wd
                .find_element(By::Css("#lblTrueAnswer"))
                .await
                .map_err(wd_error)?;
            let wrong = wd
                .find_element(By::Css("#lblFalseAnser")) // intentional typo
                .await
                .map_err(wd_error)?;
            let correct_t = correct_e.text().await.map_err(wd_error)?;
            let wrong = wrong.text().await.map_err(wd_error)?;
            correct = Some(
                num_list(&correct_t)
                    .into_iter()
                    .map(|idx| question_ids[idx - 1])
                    .collect(),
            );
            let wrong = num_list(&wrong);
            for wrong in wrong {
                unknowns.remove(&question_ids[wrong - 1]);
                println!("Incorrect question: {}", wrong);
                has_incorrect = true;
            }
        }
    }
    let unknown_questions = unknowns
        .into_iter()
        .map(|(q_id, answer_used)| {
            let answers = answer_of_questions.get(&q_id).unwrap().clone();
            (
                q_id,
                UnknownQuestion {
                    answers: answers.to_vec(),
                    answer_used,
                },
            )
        })
        .collect::<HashMap<_, _>>();
    let js_api_data = JsApiUpload {
        group: id,
        group_text: title,
        unknown_questions,
        answer_map: answer_maps,
        question_map: question_maps,
    };
    let db2 = Arc::clone(&db);
    spawn_blocking(move || actions::js_upload_call(&db2.lock().unwrap(), js_api_data)).await??;
    if let Some(correct) = correct {
        spawn_blocking(move || {
            diesel::update(answers::table.filter(answers::question_id.eq_any(&correct)))
                .set(answers::reviewed.eq(true))
                .execute(&db.lock().unwrap() as &PgConnection)
        })
        .await??;
    }
    Ok(has_incorrect && args.crack)
}

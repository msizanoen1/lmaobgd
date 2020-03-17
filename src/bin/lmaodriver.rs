use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use lmaobgd::actions;
use lmaobgd::models::*;
use lmaobgd::webdriver::*;
use rand::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use structopt::StructOpt;
use tokio::task::spawn_blocking;
use tokio::time::delay_for;

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
    /// WebDriver server URL
    webdriver_url: String,
    /// Account ID
    id: String,
    /// URL to navigate
    test_url: String,
}

#[tokio::main]
async fn main() -> Result<(), exitfailure::ExitFailure> {
    let _ = dotenv();
    let args = Args::from_args();
    let url = args.database_url;
    let db = Arc::new(Mutex::new(
        spawn_blocking(move || PgConnection::establish(&url)).await??,
    ));
    let endpoint = args.webdriver_url;
    let user = args.id;
    let test_url = args.test_url;
    let mut caps = HashMap::new();
    if args.headless {
        caps.insert(
            String::from("moz:firefoxOptions"),
            json!({
                "args": ["-headless"],
            }),
        );
    }
    let db2 = Arc::clone(&db);
    let data = spawn_blocking(move || actions::js_get_data(&db2.lock().unwrap())).await??;
    let wd = WebDriver::new(endpoint, HashMap::new(), vec![caps]).await?;
    wd.navigate("http://study.hanoi.edu.vn/dang-nhap?returnUrl=/")
        .await?;
    let username = wd.get_element(Using::CssSelector, "#UserName").await?;
    let password = wd.get_element(Using::CssSelector, "#Password").await?;
    wd.element_send_keys(&username, &user).await?;
    wd.element_send_keys(&password, &user).await?;
    let button = wd.get_element(Using::CssSelector, "#AjaxLogin").await?;
    wd.element_click(&button).await?;
    wd.navigate(&test_url).await?;
    let start = wd.get_element(Using::CssSelector, "#start-test").await?;
    wd.element_click(&start).await?;
    let title_elem = loop {
        match wd
            .get_element(Using::CssSelector, "body .row .col-12 h1")
            .await
        {
            Ok(elem) => break elem,
            _ => (),
        }
    };
    let id_elem = wd
        .get_element(Using::CssSelector, "body .row .row .col-12 div")
        .await?;
    let title = wd.get_element_text(&title_elem).await?;
    let id_str = wd.get_element_text(&id_elem).await?;
    let id = id_str
        .rsplit(':')
        .nth(0)
        .map(|x| x.trim().parse::<i32>())
        .transpose()?
        .unwrap_or(0);
    println!("Test name: {}", title);
    println!("Test ID: {}", id);
    let questions = wd.get_elements(Using::CssSelector, ".question-box").await?;
    let mut question_maps = HashMap::new();
    let mut answer_of_questions = HashMap::new();
    let mut answer_maps = HashMap::new();
    let mut unknowns = HashMap::new();
    for question in questions {
        let q_id = wd
            .get_element_attr(&question, "data-id")
            .await?
            .parse::<i32>()?;
        let cur_answer = data.get(&q_id).copied();
        if args.force_fetch || cur_answer.is_none() {
            let q_text = wd.get_element_text(&question).await?;
            println!("Question {}: {}", q_id, process_question(&q_text));
            question_maps.insert(q_id, q_text);
        }
        let inputs = wd
            .get_elements_from_element(&question, Using::CssSelector, r#"input[type="radio"]"#)
            .await?;
        let mut answers = [0; 4];
        let mut input_elems = Vec::new();
        let mut answered = false;
        for (idx, input) in inputs.into_iter().enumerate() {
            input_elems.push(input.clone());
            let a_id = wd
                .get_element_prop::<String>(&input, "value")
                .await?
                .parse::<i32>()?;
            if cur_answer.is_none() || args.force_fetch {
                let a_text = wd
                    .run_script_elem(
                        "return arguments[0].parentNode.parentNode.innerText;",
                        &input,
                    )
                    .await?;
                answer_maps.insert(a_id, a_text);
            }
            answers[idx] = a_id;
            if cur_answer == Some(a_id) {
                wd.element_send_keys(&input, "\u{e00d}").await?;
                answered = true;
            }
        }
        if !answered {
            let idx = rand::thread_rng().gen_range(0, 4);
            unknowns.insert(q_id, answers[idx]);
            wd.element_send_keys(&input_elems[idx], "\u{e00d}").await?;
        }

        answer_of_questions.insert(q_id, answers);
    }
    if !args.no_submit {
        wd.run_script_unit(r#"SendUserTestResultToServer("Đang nộp bài, vui lòng đợi và không thực hiện thêm bất cứ thao tác nào!", 2);"#).await?;
        if !args.no_autoclose {
            delay_for(Duration::from_secs(15)).await;
            wd.close().await?;
        }
    }
    let unknown_questions = unknowns
        .into_iter()
        .map(|(q_id, answer_used)| {
            let answers = *answer_of_questions.get(&q_id).unwrap();
            (
                q_id,
                UnknownQuestion {
                    answers,
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
    spawn_blocking(move || actions::js_upload_call(&db.lock().unwrap(), js_api_data)).await??;
    Ok(())
}

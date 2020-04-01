use dotenv::dotenv;
use lmaobgd::display::*;
use lmaobgd::models::*;
use lmaobgd::webdriver::*;
use rand::prelude::*;
use reqwest::Client;
use std::collections::HashMap;
use std::iter::once;
use std::mem::take;
use std::time::Duration;
use structopt::StructOpt;
use tokio::signal::ctrl_c;
use tokio::time::delay_for;

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

/// LmaoBGD WebDriver
#[derive(StructOpt)]
struct Args {
    /// The API key for API
    #[structopt(short = "k", long, env = "LMAOBGD_API_KEY", hide_env_values = true)]
    api_key: String,
    /// URL of API server
    #[structopt(short = "A", long, default_value = "http://localhost:5000/api")]
    api_url: String,
    /// Dont't auto close after 30s
    #[structopt(short, long)]
    no_autoclose: bool,
    /// Do not auto submit
    #[structopt(short = "s", long)]
    no_submit: bool,
    /// Headless mode
    #[structopt(short = "H", long)]
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
    /// Verbose mode
    #[structopt(short, long)]
    verbose: bool,
    /// Geckodriver command to use
    #[structopt(short, long)]
    geckodriver: Option<String>,
    /// Password to use (defaults to ID)
    #[structopt(short, long, env = "LMAODRIVER_PASSWORD", hide_env_values = true)]
    password: Option<String>,
    /// Account ID
    id: String,
    /// URL to navigate
    test_url: String,
}

#[tokio::main]
async fn main() -> Result<(), exitfailure::ExitFailure> {
    let _ = dotenv();
    let args = Args::from_args();
    let wd = WebDriver::new_firefox(args.geckodriver.as_ref(), args.headless, args.verbose).await?;
    let password_txt = args.password.as_ref().unwrap_or(&args.id);
    let client = Client::new();

    let main = async {
        wd.navigate("http://study.hanoi.edu.vn/dang-nhap?returnUrl=/")
            .await?;
        let username = wd.get_element(Using::CssSelector, "#UserName").await?;
        let password = wd.get_element(Using::CssSelector, "#Password").await?;
        wd.element_send_keys(&username, &args.id).await?;
        wd.element_send_keys(&password, password_txt).await?;
        let button = wd.get_element(Using::CssSelector, "#AjaxLogin").await?;
        wd.element_click(&button).await?;

        while run(&wd, &args, &args.api_url, &args.api_key, &client).await? {
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
        wd.close().await?;
    }

    Ok(())
}

async fn run(
    wd: &WebDriver,
    args: &Args,
    api: &str,
    key: &str,
    client: &Client,
) -> Result<bool, failure::Error> {
    let test_url = &args.test_url;
    let data_url = format!("{}/data", api);
    let data = client
        .get(&data_url)
        .basic_auth(key, None::<&str>)
        .send()
        .await?
        .error_for_status()?
        .json::<HashMap<i32, i32>>()
        .await?;
    wd.navigate(test_url).await?;
    let start = wd.get_element(Using::CssSelector, "#start-test").await?;
    wd.element_click(&start).await?;
    let title_elem = wd
        .get_element(Using::CssSelector, "body .row .col-12 h1")
        .await?;
    let title = wd.get_element_text(&title_elem).await?;
    println!("Test name: {}", title);
    let questions = wd.get_elements(Using::CssSelector, ".question-box").await?;
    let mut question_maps = HashMap::new();
    let mut answer_of_questions = HashMap::new();
    let mut answer_maps = HashMap::new();
    let mut unknowns = HashMap::new();
    let mut question_ids = Vec::new();
    for question in questions {
        let q_id = wd
            .get_element_attr(&question, "data-id")
            .await?
            .parse::<i32>()?;
        question_ids.push(q_id);
        let cur_answer = data.get(&q_id).copied();
        if args.force_fetch || cur_answer.is_none() {
            let q_text = wd.get_element_text(&question).await?;
            println!("Question {}: {}", q_id, process_question(&q_text));
            question_maps.insert(q_id, q_text);
        }
        let inputs = wd
            .get_elements_from_element(&question, Using::CssSelector, r#"input[type="radio"]"#)
            .await?;
        let mut answers = Vec::new();
        let mut input_elems = Vec::new();
        let mut answered = false;
        for input in inputs {
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
            answers.push(a_id);
            if cur_answer == Some(a_id) {
                wd.element_send_keys(&input, "\u{e00d}").await?;
                answered = true;
            }
        }
        if !answered {
            let idx = rand::thread_rng().gen_range(0, answers.len());
            unknowns.insert(q_id, answers[idx]);
            wd.element_send_keys(&input_elems[idx], "\u{e00d}").await?;
        }

        answer_of_questions.insert(q_id, answers);
    }
    let mut correct: Option<Vec<i32>> = None;
    let mut has_incorrect = false;
    if !args.no_submit {
        wd.run_script_unit(r#"SendUserTestResultToServer("Đang nộp bài, vui lòng đợi và không thực hiện thêm bất cứ thao tác nào!", 2);"#).await?;
        println!("Waiting for result page...");
        delay_for(Duration::from_secs(15)).await;
        if args.autoreview {
            let correct_e = wd.get_element(Using::CssSelector, "#lblTrueAnswer").await?;
            let wrong = wd
                .get_element(Using::CssSelector, "#lblFalseAnser") // intentional typo
                .await?;
            let correct_t = wd.get_element_text(&correct_e).await?;
            let wrong = wd.get_element_text(&wrong).await?;
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
        group: None,
        group_text: title,
        unknown_questions,
        answer_map: answer_maps,
        question_map: question_maps,
    };
    let upload_url = format!("{}/upload", api);
    client
        .post(&upload_url)
        .basic_auth(key, None::<&str>)
        .json(&js_api_data)
        .send()
        .await?
        .error_for_status()?;
    if let Some(correct) = correct {
        let set_correct_url = format!("{}/set_reviewed", api);
        client
            .post(&set_correct_url)
            .basic_auth(key, None::<&str>)
            .json(&correct)
            .send()
            .await?
            .error_for_status()?;
    }
    Ok(has_incorrect && args.crack)
}

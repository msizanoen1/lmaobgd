use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use lmaobgd::actions;
use lmaobgd::models::*;
use lmaobgd::webdriver::*;
use rand::prelude::*;
use std::collections::HashMap;
use structopt::StructOpt;

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
    /// WebDriver server URL
    webdriver_url: String,
    /// Account ID
    id: String,
    /// URL to navigate
    test_url: String,
}

fn main() -> Result<(), exitfailure::ExitFailure> {
    let _ = dotenv();
    let args = Args::from_args();
    let db = PgConnection::establish(&args.database_url)?;
    let endpoint = args.webdriver_url;
    let user = args.id;
    let test_url = args.test_url;
    let wd = WebDriver::new(endpoint, HashMap::new(), Vec::new())?;
    wd.navigate("http://study.hanoi.edu.vn/dang-nhap?returnUrl=/")?;
    let username = wd.get_element(Using::CssSelector, "#UserName")?;
    let password = wd.get_element(Using::CssSelector, "#Password")?;
    wd.element_send_keys(&username, &user)?;
    wd.element_send_keys(&password, &user)?;
    let button = wd.get_element(Using::CssSelector, "#AjaxLogin")?;
    wd.element_click(&button)?;
    wd.navigate(&test_url)?;
    let start = wd.get_element(Using::CssSelector, "#start-test")?;
    wd.element_click(&start)?;
    let title_elem = wd.get_element(Using::CssSelector, "body .row .col-12 h1")?;
    let id_elem = wd.get_element(Using::CssSelector, "body .row .row .col-12 div")?;
    let title = wd.get_element_text(&title_elem)?;
    let id_str = wd.get_element_text(&id_elem)?;
    let id = id_str
        .rsplit(':')
        .nth(0)
        .map(|x| x.trim().parse::<i32>())
        .transpose()?
        .unwrap_or(0);
    let questions = wd.get_elements(Using::CssSelector, ".question-box")?;
    let mut question_maps = HashMap::new();
    let mut answer_maps = HashMap::new();
    let mut answer_of_questions = HashMap::new();
    for question in questions {
        let q_id = wd.get_element_attr(&question, "data-id")?.parse::<i32>()?;
        let q_text = wd.get_element_text(&question)?;
        question_maps.insert(q_id, q_text);
        let inputs =
            wd.get_elements_from_element(&question, Using::CssSelector, r#"input[type="radio"]"#)?;
        let mut answers = [0; 4];
        for (idx, input) in inputs.into_iter().enumerate() {
            let a_id = wd.get_element_attr(&input, "value")?.parse::<i32>()?;
            let a_text = wd.run_script_elem(
                "return arguments[0].parentNode.parentNode.innerText;",
                &input,
            )?;
            answer_maps.insert(a_id, a_text);
            answers[idx] = a_id;
        }
        answer_of_questions.insert(q_id, answers);
    }
    let data = actions::js_get_data(&db)?;
    let mut unknowns = HashMap::new();
    for (q_id, answers) in &answer_of_questions {
        let cur_answer = match data.get(&q_id) {
            Some(a) => *a,
            None => {
                let idx = rand::thread_rng().gen_range(0, 4);
                let a = answers[idx];
                unknowns.insert(q_id, a);
                a
            }
        };
        let radio = wd.get_element(
            Using::CssSelector,
            format!(
                r#".question-box[data-id="{data_id}"] input[value="{answer}"]"#,
                data_id = q_id,
                answer = cur_answer
            ),
        )?;
        wd.element_click(&radio)?;
    }
    if !args.no_submit {
        wd.run_script_unit(r#"SendUserTestResultToServer("Đang nộp bài, vui lòng đợi và không thực hiện thêm bất cứ thao tác nào!", 2);"#)?;
        if !args.no_autoclose {
            std::thread::sleep(std::time::Duration::from_secs(30));
            wd.close()?;
        }
    }
    let unknown_questions = unknowns
        .into_iter()
        .map(|(q_id, answer_used)| {
            let answers = *answer_of_questions.get(&q_id).unwrap();
            (
                *q_id,
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
    actions::js_upload_call(&db, js_api_data)?;
    Ok(())
}

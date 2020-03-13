use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use failure::Error;
use lmaobgd::actions;
use lmaobgd::models::*;
use lmaobgd::webdriver::*;
use rand::prelude::*;
use std::collections::HashMap;
use std::env;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Error> {
    let _ = dotenv();
    let dbu = env::var("DATABASE_URL")?;
    let db = PgConnection::establish(&dbu)?;
    let mut args = env::args().skip(1);
    let endpoint = args.next().unwrap();
    let user = args.next().unwrap();
    let test_url = args.next().unwrap();
    let wd = WebDriver::new(endpoint, HashMap::new(), Vec::new())?;
    wd.navigate("http://study.hanoi.edu.vn/dang-nhap?returnUrl=/")?;
    thread::sleep(Duration::from_secs(2));
    let username = wd.get_element(Using::CssSelector, "#UserName")?;
    let password = wd.get_element(Using::CssSelector, "#Password")?;
    wd.element_send_keys(&username, &user)?;
    wd.element_send_keys(&password, &user)?;
    let button = wd.get_element(Using::CssSelector, "#AjaxLogin")?;
    wd.element_click(&button)?;
    thread::sleep(Duration::from_secs(2));
    wd.navigate(&test_url)?;
    thread::sleep(Duration::from_secs(2));
    let start = wd.get_element(Using::CssSelector, "#start-test")?;
    wd.element_click(&start)?;
    thread::sleep(Duration::from_secs(2));
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
            let a_text = wd.get_element_text(&input)?;
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

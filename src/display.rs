pub fn process_question(q: &str) -> String {
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

pub fn process_question2(q: &str) -> String {
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
    vec.truncate(vec.len().saturating_sub(n));
    let iter = vec.into_iter().skip(1);
    format!("{}", iter.collect::<Vec<_>>().join("\n"))
}

pub fn process_answer(s: &str) -> String {
    let mut iter = s.lines().map(|s| s.trim()).filter(|s| *s != "");
    format!(
        "{} {}",
        iter.next().unwrap_or(""),
        iter.collect::<Vec<_>>().join("\n")
    )
}

pub fn process_answer2(s: &str) -> String {
    let iter = s.lines().map(|s| s.trim()).filter(|s| *s != "").skip(1);
    format!("{}", iter.collect::<Vec<_>>().join("\n"))
}

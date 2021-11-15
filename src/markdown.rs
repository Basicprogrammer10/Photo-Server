// 5-Star Markdown Rendering

pub fn render(markdown: String) -> String {
    let markdown = match render_bold_italic(markdown.clone()) {
        Some(i) => i,
        None => markdown,
    };

    let mut new = String::new();
    for i in markdown.lines() {
        if i.is_empty() {
            new.push_str("</br>");
            continue;
        }

        if i.starts_with("# ") {
            new.push_str("<h1>");
            new.push_str(&get_after_sep("# ", i));
            new.push_str("</h1>");
            continue;
        }

        if i.starts_with("## ") {
            new.push_str("<h2>");
            new.push_str(&get_after_sep("## ", i));
            new.push_str("</h2>");
            continue;
        }

        if i.starts_with("### ") {
            new.push_str("<h3>");
            new.push_str(&get_after_sep("### ", i));
            new.push_str("</h3>");
            continue;
        }

        if i.starts_with("#### ") {
            new.push_str("<h4>");
            new.push_str(&get_after_sep("#### ", i));
            new.push_str("</h4>");
            continue;
        }

        if i.starts_with("##### ") {
            new.push_str("<h5>");
            new.push_str(&get_after_sep("##### ", i));
            new.push_str("</h5>");
            continue;
        }

        if i.starts_with("###### ") {
            new.push_str("<h6>");
            new.push_str(&get_after_sep("###### ", i));
            new.push_str("</h6>");
            continue;
        }

        new.push_str("<p>");
        new.push_str(i);
        new.push_str("</p>");
    }

    new
}

fn get_after_sep(sep: &str, content: &str) -> String {
    let mut parts = content.split(sep).collect::<Vec<&str>>();
    parts.remove(0);

    parts.join(sep)
}

fn render_bold_italic(markdown: String) -> Option<String> {
    let ch = markdown.chars();
    let mut strong = false;
    let mut italic = false;
    let mut new = String::new();

    for i in 0..markdown.len() - 1 {
        if ch.clone().nth(i)? == '*' && ch.clone().nth(i + 1)? == '*' {
            if strong {
                new.push_str("</strong>");
                strong = false;
                continue;
            }
            new.push_str("<strong>");
            strong = true;
            continue;
        }

        if ch.clone().nth(i)? == '*' {
            if italic {
                new.push_str("</em>");
                italic = false;
                continue;
            }
            new.push_str("<em>");
            italic = true;
            continue;
        }

        new.push(ch.clone().nth(i)?);
    }

    Some(new)
}

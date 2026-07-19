pub fn render_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut code_content = String::new();

    for line in markdown.lines() {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("<pre><code");
                if !code_lang.is_empty() {
                    html.push_str(&format!(" class=\"language-{}\"", escape_html(&code_lang)));
                }
                html.push('>');
                html.push_str(&escape_html(&code_content));
                html.push_str("</code></pre>\n");
                code_content.clear();
                code_lang.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
                code_lang = line.trim_start_matches("```").trim().to_string();
            }
            continue;
        }

        if in_code_block {
            code_content.push_str(line);
            code_content.push('\n');
            continue;
        }

        if line.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", escape_html(&line[2..])));
        } else if line.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", escape_html(&line[3..])));
        } else if line.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", escape_html(&line[4..])));
        } else if line.starts_with("- ") || line.starts_with("* ") {
            html.push_str(&format!("<li>{}</li>\n", escape_html(&line[2..])));
        } else if line.starts_with("> ") {
            html.push_str(&format!("<blockquote>{}</blockquote>\n", escape_html(&line[2..])));
        } else if line.trim().is_empty() {
            html.push_str("<br>\n");
        } else {
            html.push_str(&format!("<p>{}</p>\n", escape_html(line)));
        }
    }

    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

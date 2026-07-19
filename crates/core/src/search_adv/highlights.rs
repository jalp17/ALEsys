use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightMatch {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightedText {
    pub original: String,
    pub highlighted: String,
    pub matches: Vec<HighlightMatch>,
}

pub struct Highlighter {
    tag_open: String,
    tag_close: String,
}

impl Highlighter {
    pub fn new() -> Self {
        Self {
            tag_open: "<mark>".to_string(),
            tag_close: "</mark>".to_string(),
        }
    }

    pub fn with_tags(open: &str, close: &str) -> Self {
        Self {
            tag_open: open.to_string(),
            tag_close: close.to_string(),
        }
    }

    pub fn highlight(&self, text: &str, query: &str) -> HighlightedText {
        if query.is_empty() {
            return HighlightedText {
                original: text.to_string(),
                highlighted: text.to_string(),
                matches: vec![],
            };
        }

        let mut matches = Vec::new();
        let text_lower = text.to_lowercase();
        let query_lower = query.to_lowercase();

        let mut start = 0;
        while let Some(pos) = text_lower[start..].find(&query_lower) {
            let absolute_start = start + pos;
            let absolute_end = absolute_start + query.len();

            matches.push(HighlightMatch {
                start: absolute_start,
                end: absolute_end,
                text: text[absolute_start..absolute_end].to_string(),
            });

            start = absolute_end;
        }

        let mut highlighted = String::new();
        let mut last_end = 0;

        for m in &matches {
            highlighted.push_str(&text[last_end..m.start]);
            highlighted.push_str(&self.tag_open);
            highlighted.push_str(&m.text);
            highlighted.push_str(&self.tag_close);
            last_end = m.end;
        }

        highlighted.push_str(&text[last_end..]);

        HighlightedText {
            original: text.to_string(),
            highlighted,
            matches,
        }
    }

    pub fn highlight_multi(&self, text: &str, queries: &[String]) -> HighlightedText {
        if queries.is_empty() {
            return self.highlight(text, "");
        }

        let mut combined = text.to_string();
        let mut all_matches = Vec::new();

        for query in queries {
            let result = self.highlight(&combined, query);
            combined = result.highlighted.clone();
            all_matches.extend(result.matches);
        }

        HighlightedText {
            original: text.to_string(),
            highlighted: combined,
            matches: all_matches,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_basic() {
        let highlighter = Highlighter::new();
        let result = highlighter.highlight("Hello World", "World");
        assert!(result.highlighted.contains("<mark>World</mark>"));
        assert_eq!(result.matches.len(), 1);
    }

    #[test]
    fn test_highlight_case_insensitive() {
        let highlighter = Highlighter::new();
        let result = highlighter.highlight("Hello World", "hello");
        assert!(result.highlighted.contains("<mark>Hello</mark>"));
    }

    #[test]
    fn test_highlight_empty_query() {
        let highlighter = Highlighter::new();
        let result = highlighter.highlight("Hello World", "");
        assert_eq!(result.highlighted, "Hello World");
        assert!(result.matches.is_empty());
    }

    #[test]
    fn test_highlight_multiple_matches() {
        let highlighter = Highlighter::new();
        let result = highlighter.highlight("test test test", "test");
        assert_eq!(result.matches.len(), 3);
    }

    #[test]
    fn test_highlight_custom_tags() {
        let highlighter = Highlighter::with_tags("**", "**");
        let result = highlighter.highlight("Hello World", "World");
        assert!(result.highlighted.contains("**World**"));
    }
}
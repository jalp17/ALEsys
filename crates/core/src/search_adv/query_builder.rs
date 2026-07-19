use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub text: String,
    pub filters: Vec<super::filters::SearchFilter>,
    pub facets: Vec<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub page: usize,
    pub page_size: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: String::new(),
            filters: vec![],
            facets: vec![],
            sort_by: None,
            sort_order: None,
            page: 0,
            page_size: 20,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub results: Vec<SearchItem>,
    pub total: usize,
    pub page: usize,
    pub page_size: usize,
    pub query_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub score: f64,
    pub highlights: Vec<super::highlights::HighlightMatch>,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct QueryBuilder {
    synonyms: std::collections::HashMap<String, Vec<String>>,
}

impl QueryBuilder {
    pub fn new() -> Self {
        let mut synonyms = std::collections::HashMap::new();
        synonyms.insert("bug".to_string(), vec!["error".to_string(), "issue".to_string(), "defect".to_string()]);
        synonyms.insert("fix".to_string(), vec!["resolve".to_string(), "patch".to_string(), "repair".to_string()]);
        synonyms.insert("test".to_string(), vec!["spec".to_string(), "check".to_string(), "verify".to_string()]);

        Self { synonyms }
    }

    pub fn expand_query(&self, query: &str) -> Vec<String> {
        let mut terms: Vec<String> = query.split_whitespace().map(String::from).collect();

        for term in &terms.clone() {
            if let Some(syns) = self.synonyms.get(term.as_str()) {
                for syn in syns {
                    terms.push(syn.clone());
                }
            }
        }

        terms
    }

    pub fn search(&self, query: &SearchQuery, documents: &[SearchItem]) -> QueryResult {
        let start = std::time::Instant::now();

        let expanded_terms = self.expand_query(&query.text);
        let query_lower = query.text.to_lowercase();

        let mut results: Vec<SearchItem> = documents.iter()
            .filter(|doc| {
                let title_lower = doc.title.to_lowercase();
                let content_lower = doc.content.to_lowercase();

                let text_match = query_lower.is_empty()
                    || title_lower.contains(&query_lower)
                    || content_lower.contains(&query_lower);

                let expanded_match = expanded_terms.iter().any(|term| {
                    let term_lower = term.to_lowercase();
                    title_lower.contains(&term_lower) || content_lower.contains(&term_lower)
                });

                text_match || expanded_match
            })
            .cloned()
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let total = results.len();
        let start_idx = query.page * query.page_size;
        let paged: Vec<SearchItem> = results.into_iter().skip(start_idx).take(query.page_size).collect();

        QueryResult {
            results: paged,
            total,
            page: query.page,
            page_size: query.page_size,
            query_time_ms: start.elapsed().as_millis() as u64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(id: &str, title: &str, content: &str) -> SearchItem {
        SearchItem {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            score: 1.0,
            highlights: vec![],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_expand_query() {
        let builder = QueryBuilder::new();
        let terms = builder.expand_query("bug fix");
        assert!(terms.contains(&"bug".to_string()));
        assert!(terms.contains(&"error".to_string()));
        assert!(terms.contains(&"fix".to_string()));
        assert!(terms.contains(&"resolve".to_string()));
    }

    #[test]
    fn test_search_basic() {
        let builder = QueryBuilder::new();
        let docs = vec![
            make_item("1", "Rust Programming", "Learn Rust"),
            make_item("2", "Python Guide", "Learn Python"),
        ];
        let query = SearchQuery { text: "rust".to_string(), ..Default::default() };
        let result = builder.search(&query, &docs);
        assert_eq!(result.total, 1);
    }

    #[test]
    fn test_search_empty_query() {
        let builder = QueryBuilder::new();
        let docs = vec![
            make_item("1", "Doc 1", "Content 1"),
            make_item("2", "Doc 2", "Content 2"),
        ];
        let query = SearchQuery { text: String::new(), ..Default::default() };
        let result = builder.search(&query, &docs);
        assert_eq!(result.total, 2);
    }

    #[test]
    fn test_search_synonyms() {
        let builder = QueryBuilder::new();
        let docs = vec![
            make_item("1", "Bug Report", "Found a bug in the code"),
            make_item("2", "Feature", "New feature request"),
        ];
        let query = SearchQuery { text: "bug".to_string(), ..Default::default() };
        let result = builder.search(&query, &docs);
        assert!(result.total >= 1);
    }

    #[test]
    fn test_search_pagination() {
        let builder = QueryBuilder::new();
        let docs: Vec<SearchItem> = (0..25).map(|i| make_item(&i.to_string(), &format!("Doc {}", i), "Content")).collect();
        let query = SearchQuery { text: String::new(), page: 0, page_size: 10, ..Default::default() };
        let result = builder.search(&query, &docs);
        assert_eq!(result.results.len(), 10);
        assert_eq!(result.total, 25);
    }
}
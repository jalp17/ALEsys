use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FilterType {
    Text,
    Date,
    Type,
    Tag,
    Range,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilter {
    pub field: String,
    pub filter_type: FilterType,
    pub value: String,
    pub operator: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterGroup {
    pub logic: String,
    pub filters: Vec<SearchFilter>,
}

impl SearchFilter {
    pub fn text(field: &str, value: &str) -> Self {
        Self {
            field: field.to_string(),
            filter_type: FilterType::Text,
            value: value.to_string(),
            operator: "contains".to_string(),
        }
    }

    pub fn date_range(field: &str, start: &str, end: &str) -> Self {
        Self {
            field: field.to_string(),
            filter_type: FilterType::Date,
            value: format!("{}..{}", start, end),
            operator: "between".to_string(),
        }
    }

    pub fn tag(tag: &str) -> Self {
        Self {
            field: "tags".to_string(),
            filter_type: FilterType::Tag,
            value: tag.to_string(),
            operator: "has".to_string(),
        }
    }

    pub fn doc_type(doc_type: &str) -> Self {
        Self {
            field: "type".to_string(),
            filter_type: FilterType::Type,
            value: doc_type.to_string(),
            operator: "equals".to_string(),
        }
    }

    pub fn matches(&self, item: &std::collections::HashMap<String, String>) -> bool {
        match self.filter_type {
            FilterType::Text => {
                item.get(&self.field)
                    .map(|v| v.to_lowercase().contains(&self.value.to_lowercase()))
                    .unwrap_or(false)
            }
            FilterType::Tag => {
                item.get("tags")
                    .map(|v| v.split(',').any(|t| t.trim() == self.value))
                    .unwrap_or(false)
            }
            FilterType::Type => {
                item.get(&self.field)
                    .map(|v| v == &self.value)
                    .unwrap_or(false)
            }
            FilterType::Date => true,
            FilterType::Range => true,
        }
    }
}

impl FilterGroup {
    pub fn and(filters: Vec<SearchFilter>) -> Self {
        Self {
            logic: "and".to_string(),
            filters,
        }
    }

    pub fn or(filters: Vec<SearchFilter>) -> Self {
        Self {
            logic: "or".to_string(),
            filters,
        }
    }

    pub fn matches(&self, item: &std::collections::HashMap<String, String>) -> bool {
        if self.filters.is_empty() {
            return true;
        }

        match self.logic.as_str() {
            "and" => self.filters.iter().all(|f| f.matches(item)),
            "or" => self.filters.iter().any(|f| f.matches(item)),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_filter() {
        let filter = SearchFilter::text("title", "rust");
        let mut item = std::collections::HashMap::new();
        item.insert("title".to_string(), "Rust Programming".to_string());
        assert!(filter.matches(&item));
    }

    #[test]
    fn test_text_filter_no_match() {
        let filter = SearchFilter::text("title", "python");
        let mut item = std::collections::HashMap::new();
        item.insert("title".to_string(), "Rust Programming".to_string());
        assert!(!filter.matches(&item));
    }

    #[test]
    fn test_tag_filter() {
        let filter = SearchFilter::tag("code");
        let mut item = std::collections::HashMap::new();
        item.insert("tags".to_string(), "rust,code,tutorial".to_string());
        assert!(filter.matches(&item));
    }

    #[test]
    fn test_type_filter() {
        let filter = SearchFilter::doc_type("article");
        let mut item = std::collections::HashMap::new();
        item.insert("type".to_string(), "article".to_string());
        assert!(filter.matches(&item));
    }

    #[test]
    fn test_filter_group_and() {
        let group = FilterGroup::and(vec![
            SearchFilter::text("title", "rust"),
            SearchFilter::tag("code"),
        ]);
        let mut item = std::collections::HashMap::new();
        item.insert("title".to_string(), "Rust Programming".to_string());
        item.insert("tags".to_string(), "rust,code".to_string());
        assert!(group.matches(&item));
    }

    #[test]
    fn test_filter_group_or() {
        let group = FilterGroup::or(vec![
            SearchFilter::text("title", "rust"),
            SearchFilter::text("title", "python"),
        ]);
        let mut item = std::collections::HashMap::new();
        item.insert("title".to_string(), "Python Guide".to_string());
        assert!(group.matches(&item));
    }
}
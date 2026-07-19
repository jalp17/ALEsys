use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    pub field: String,
    pub values: Vec<FacetValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacetValue {
    pub value: String,
    pub count: usize,
}

pub struct FacetedSearch;

impl FacetedSearch {
    pub fn new() -> Self {
        Self
    }

    pub fn compute_facets(&self, items: &[std::collections::HashMap<String, String>], fields: &[String]) -> Vec<Facet> {
        fields.iter().map(|field| {
            let mut counts = std::collections::HashMap::new();

            for item in items {
                if let Some(value) = item.get(field.as_str()) {
                    if field == "tags" {
                        for tag in value.split(',') {
                            let tag = tag.trim().to_string();
                            *counts.entry(tag).or_insert(0) += 1;
                        }
                    } else {
                        *counts.entry(value.clone()).or_insert(0) += 1;
                    }
                }
            }

            let mut values: Vec<FacetValue> = counts.into_iter()
                .map(|(value, count)| FacetValue { value, count })
                .collect();
            values.sort_by(|a, b| b.count.cmp(&a.count));

            Facet {
                field: field.clone(),
                values,
            }
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_facets() {
        let search = FacetedSearch::new();
        let items = vec![
            [("type".to_string(), "article".to_string())].iter().cloned().collect(),
            [("type".to_string(), "article".to_string())].iter().cloned().collect(),
            [("type".to_string(), "video".to_string())].iter().cloned().collect(),
        ];
        let facets = search.compute_facets(&items, &["type".to_string()]);
        assert_eq!(facets.len(), 1);
        assert_eq!(facets[0].values.len(), 2);
    }

    #[test]
    fn test_facet_tags() {
        let search = FacetedSearch::new();
        let items = vec![
            [("tags".to_string(), "rust,code".to_string())].iter().cloned().collect(),
            [("tags".to_string(), "rust,tutorial".to_string())].iter().cloned().collect(),
        ];
        let facets = search.compute_facets(&items, &["tags".to_string()]);
        let rust_facet = &facets[0].values.iter().find(|v| v.value == "rust").unwrap();
        assert_eq!(rust_facet.count, 2);
    }

    #[test]
    fn test_empty_facets() {
        let search = FacetedSearch::new();
        let facets = search.compute_facets(&[], &["type".to_string()]);
        assert!(facets[0].values.is_empty());
    }
}
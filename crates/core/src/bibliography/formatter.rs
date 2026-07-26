//! Citation formatter - TICKET-30.3
//! APA 7, MLA 9, Chicago, IEEE, BibTeX output formats

use crate::bibliography::{Citation, CitationStyle};

pub struct CitationFormatter;

#[derive(Debug)]
pub enum FormatError {
    UnsupportedStyle(CitationStyle),
    MissingField(&'static str),
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatError::UnsupportedStyle(style) => {
                write!(f, "Unsupported citation style: {:?}", style)
            }
            FormatError::MissingField(field) => write!(f, "Missing required field: {}", field),
        }
    }
}

impl std::error::Error for FormatError {}

impl CitationFormatter {
    pub fn format(citation: &Citation, style: CitationStyle) -> Result<String, FormatError> {
        match style {
            CitationStyle::APA => Ok(Self::format_apa(citation)),
            CitationStyle::MLA => Ok(Self::format_mla(citation)),
            CitationStyle::Chicago => Ok(Self::format_chicago(citation)),
            CitationStyle::IEEE => Ok(Self::format_ieee(citation)),
            CitationStyle::Unknown => Err(FormatError::UnsupportedStyle(style)),
        }
    }

    pub fn format_apa(citation: &Citation) -> String {
        let authors = if citation.authors.is_empty() {
            String::new()
        } else if citation.authors.len() == 1 {
            format!("{}.", Self::format_author_apa(&citation.authors[0]))
        } else if citation.authors.len() == 2 {
            format!("{}, & {}.", Self::format_author_apa(&citation.authors[0]), Self::format_author_apa(&citation.authors[1]))
        } else {
            let formatted: Vec<String> = citation.authors.iter().take(19).map(|a| Self::format_author_apa(a)).collect();
            format!("{}, ... {}, ({}).", formatted[0], formatted.last().unwrap_or(&String::new()), citation.year.unwrap_or(0))
        };

        let year = citation.year.map(|y| format!("({}).", y)).unwrap_or_default();
        let title = citation.title.as_ref().map(|t| format!(" {}", t)).unwrap_or_default();
        let journal = citation.journal.as_ref().map(|j| format!(" {}", j)).unwrap_or_default();
        let volume = citation.volume.as_ref().map(|v| format!(", {}", v)).unwrap_or_default();
        let issue = citation.issue.as_ref().map(|i| format!("({})", i)).unwrap_or_default();
        let pages = citation.pages.as_ref().map(|p| format!(", {}", p)).unwrap_or_default();
        let doi = citation.doi.as_ref().map(|d| format!(" https://doi.org/{}", d)).unwrap_or_default();

        format!("{}{}{}{}{}{}{}{}.", authors, year, title, journal, volume, issue, pages, doi)
    }

    pub fn format_mla(citation: &Citation) -> String {
        let author = if !citation.authors.is_empty() {
            Self::format_author_mla(&citation.authors[0])
        } else {
            String::new()
        };

        let title = citation.title.as_ref().map(|t| format!("\"{}\"", t)).unwrap_or_default();
        let journal = citation.journal.as_ref().map(|j| format!(" {}", j)).unwrap_or_default();
        let volume = citation.volume.as_ref().map(|v| format!(", vol. {}", v)).unwrap_or_default();
        let issue = citation.issue.as_ref().map(|i| format!(", no. {}", i)).unwrap_or_default();
        let year = citation.year.map(|y| format!(", {}", y)).unwrap_or_default();
        let pages = citation.pages.as_ref().map(|p| format!(", pp. {}", p)).unwrap_or_default();

        format!("{}.{}.{}{}{}{}{}.", author, title, journal, volume, issue, year, pages)
    }

    pub fn format_chicago(citation: &Citation) -> String {
        let author = if !citation.authors.is_empty() {
            Self::format_author_chicago(&citation.authors[0])
        } else {
            String::new()
        };

        let title = citation.title.as_ref().map(|t| format!("\"{},\"", t)).unwrap_or_default();
        let journal = citation.journal.as_ref().map(|j| format!(" {}", j)).unwrap_or_default();
        let volume = citation.volume.as_ref().map(|v| format!(" {}", v)).unwrap_or_default();
        let issue = citation.issue.as_ref().map(|i| format!(", no. {}", i)).unwrap_or_default();
        let year = citation.year.map(|y| format!(" ({}): ", y)).unwrap_or_default();
        let pages = citation.pages.as_ref().map(|p| format!("{}", p)).unwrap_or_default();
        let doi = citation.doi.as_ref().map(|d| format!(" https://doi.org/{}", d)).unwrap_or_default();

        format!("{}{}{}{}{}{}{}{}.", author, title, journal, volume, issue, year, pages, doi)
    }

    pub fn format_ieee(citation: &Citation) -> String {
        let author = if !citation.authors.is_empty() {
            citation.authors.join(", ")
        } else {
            String::new()
        };

        let title = citation.title.as_ref().map(|t| format!(" \"{}\"", t)).unwrap_or_default();
        let journal = citation.journal.as_ref().map(|j| format!(", {}", j)).unwrap_or_default();
        let volume = citation.volume.as_ref().map(|v| format!(", vol. {}", v)).unwrap_or_default();
        let number = citation.issue.as_ref().map(|n| format!(", no. {}", n)).unwrap_or_default();
        let year = citation.year.map(|y| format!(", {}", y)).unwrap_or_default();
        let pages = citation.pages.as_ref().map(|p| format!(", pp. {}", p)).unwrap_or_default();

        format!("{}.{}{}{}{}{}{}.", author, title, journal, volume, number, year, pages)
    }

    pub fn format_bibtex(citation: &Citation) -> String {
        let key = format!(
            "{}{}",
            citation.authors.first().map(|a| a.chars().take(3).collect::<String>()).unwrap_or_default(),
            citation.year.unwrap_or(0)
        );

        let mut entry = format!("@article{{{},\n", key);
        entry.push_str(&format!("  author = {{{}}},\n", citation.authors.join(" and ")));
        if let Some(ref title) = citation.title {
            entry.push_str(&format!("  title = {{{}}},\n", title));
        }
        if let Some(ref journal) = citation.journal {
            entry.push_str(&format!("  journal = {{{}}},\n", journal));
        }
        if let Some(year) = citation.year {
            entry.push_str(&format!("  year = {{{}}},\n", year));
        }
        if let Some(ref doi) = citation.doi {
            entry.push_str(&format!("  doi = {{{}}},\n", doi));
        }
        entry.push_str("}");

        entry
    }

    fn format_author_apa(author: &str) -> String {
        let parts: Vec<&str> = author.split(',').collect();
        if parts.len() == 2 {
            format!("{}, {}", parts[0].trim(), parts[1].trim())
        } else {
            author.to_string()
        }
    }

    fn format_author_mla(author: &str) -> String {
        let parts: Vec<&str> = author.split(',').collect();
        if parts.len() == 2 {
            format!("{} {}", parts[1].trim(), parts[0].trim())
        } else {
            author.to_string()
        }
    }

    fn format_author_chicago(author: &str) -> String {
        let parts: Vec<&str> = author.split(',').collect();
        if parts.len() == 2 {
            format!("{} {}", parts[1].trim(), parts[0].trim())
        } else {
            author.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bibliography::Citation;

    fn sample_citation() -> Citation {
        let mut c = Citation::new("Smith, J. (2023). The Future of AI. Nature, 612(3), 45-67.".to_string(), 1);
        c.title = Some("The Future of AI".to_string());
        c.authors = vec!["Smith, John".to_string()];
        c.journal = Some("Nature".to_string());
        c.year = Some(2023);
        c.volume = Some("612".to_string());
        c.issue = Some("3".to_string());
        c.pages = Some("45-67".to_string());
        c.doi = Some("10.1234/test".to_string());
        c
    }

    #[test]
    fn test_apa_format() {
        let c = sample_citation();
        let formatted = CitationFormatter::format_apa(&c);
        assert!(formatted.contains("Smith, John"));
        assert!(formatted.contains("2023"));
        assert!(formatted.contains("The Future of AI"));
        assert!(formatted.contains("Nature"));
        assert!(formatted.contains("612"));
        assert!(formatted.contains("doi"));
    }

    #[test]
    fn test_mla_format() {
        let c = sample_citation();
        let formatted = CitationFormatter::format_mla(&c);
        assert!(formatted.contains("John Smith") || formatted.contains("Smith, John"));
        assert!(formatted.contains("\"The Future of AI\""));
        assert!(formatted.contains("Nature"));
    }

    #[test]
    fn test_chicago_format() {
        let c = sample_citation();
        let formatted = CitationFormatter::format_chicago(&c);
        assert!(formatted.contains("The Future of AI"));
        assert!(formatted.contains("Nature"));
        assert!(formatted.contains("vol.") || formatted.contains("612"));
    }

    #[test]
    fn test_ieee_format() {
        let c = sample_citation();
        let formatted = CitationFormatter::format_ieee(&c);
        assert!(formatted.contains("Smith, John"));
        assert!(formatted.contains("\"The Future of AI\""));
        assert!(formatted.contains("Nature"));
    }

    #[test]
    fn test_bibtex_format() {
        let c = sample_citation();
        let formatted = CitationFormatter::format_bibtex(&c);
        assert!(formatted.starts_with("@article{"));
        assert!(formatted.contains("author = "));
        assert!(formatted.contains("title = "));
        assert!(formatted.contains("journal = "));
        assert!(formatted.contains("year = "));
        assert!(formatted.contains("doi = "));
    }

    #[test]
    fn test_unknown_style_returns_error() {
        let c = sample_citation();
        let result = CitationFormatter::format(&c, CitationStyle::Unknown);
        assert!(result.is_err());
    }

    #[test]
    fn test_author_reformat_apa() {
        let parts = vec!["Smith", "John"];
        let formatted = CitationFormatter::format_author_apa(&parts.join(", "));
        assert_eq!(formatted, "Smith, John");
    }
}

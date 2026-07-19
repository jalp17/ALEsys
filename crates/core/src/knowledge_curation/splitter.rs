use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitStrategy {
    BySize { max_chars: usize },
    ByHeaders,
    ByParagraphs,
    BySentences { max_sentences: usize },
    Smart,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitResult {
    pub chunks: Vec<DocumentChunk>,
    pub original_id: String,
    pub strategy_used: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: String,
    pub content: String,
    pub index: usize,
    pub start_offset: usize,
    pub end_offset: usize,
    pub metadata: std::collections::HashMap<String, String>,
}

pub struct DocumentSplitter {
    strategy: SplitStrategy,
}

impl DocumentSplitter {
    pub fn new(strategy: SplitStrategy) -> Self {
        Self { strategy }
    }

    pub fn split(&self, document_id: &str, content: &str) -> SplitResult {
        let chunks = match &self.strategy {
            SplitStrategy::BySize { max_chars } => self.split_by_size(content, *max_chars),
            SplitStrategy::ByHeaders => self.split_by_headers(content),
            SplitStrategy::ByParagraphs => self.split_by_paragraphs(content),
            SplitStrategy::BySentences { max_sentences } => {
                self.split_by_sentences(content, *max_sentences)
            }
            SplitStrategy::Smart => self.smart_split(content),
        };

        SplitResult {
            chunks,
            original_id: document_id.to_string(),
            strategy_used: format!("{:?}", self.strategy),
            warnings: vec![],
        }
    }

    fn split_by_size(&self, content: &str, max_chars: usize) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < content.len() {
            let end = std::cmp::min(start + max_chars, content.len());
            let chunk_content = content[start..end].to_string();

            chunks.push(DocumentChunk {
                id: format!("chunk-{}", index),
                content: chunk_content,
                index,
                start_offset: start,
                end_offset: end,
                metadata: std::collections::HashMap::new(),
            });

            start = end;
            index += 1;
        }

        chunks
    }

    fn split_by_headers(&self, content: &str) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut start = 0;
        let mut index = 0;

        for line in content.lines() {
            if line.starts_with('#') && !current_chunk.is_empty() {
                chunks.push(DocumentChunk {
                    id: format!("chunk-{}", index),
                    content: current_chunk.clone(),
                    index,
                    start_offset: start,
                    end_offset: start + current_chunk.len(),
                    metadata: std::collections::HashMap::new(),
                });
                index += 1;
                current_chunk.clear();
                start += current_chunk.len();
            }
            current_chunk.push_str(line);
            current_chunk.push('\n');
        }

        if !current_chunk.is_empty() {
            chunks.push(DocumentChunk {
                id: format!("chunk-{}", index),
                content: current_chunk,
                index,
                start_offset: start,
                end_offset: content.len(),
                metadata: std::collections::HashMap::new(),
            });
        }

        chunks
    }

    fn split_by_paragraphs(&self, content: &str) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut start = 0;
        let mut index = 0;

        for line in content.lines() {
            if line.trim().is_empty() && !current_chunk.is_empty() {
                chunks.push(DocumentChunk {
                    id: format!("chunk-{}", index),
                    content: current_chunk.clone(),
                    index,
                    start_offset: start,
                    end_offset: start + current_chunk.len(),
                    metadata: std::collections::HashMap::new(),
                });
                index += 1;
                current_chunk.clear();
                start += current_chunk.len();
            }
            current_chunk.push_str(line);
            current_chunk.push('\n');
        }

        if !current_chunk.is_empty() {
            chunks.push(DocumentChunk {
                id: format!("chunk-{}", index),
                content: current_chunk,
                index,
                start_offset: start,
                end_offset: content.len(),
                metadata: std::collections::HashMap::new(),
            });
        }

        chunks
    }

    fn split_by_sentences(&self, content: &str, max_sentences: usize) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let sentences: Vec<&str> = content.split(|c| c == '.' || c == '!' || c == '?').collect();
        let mut current_chunk = String::new();
        let mut start = 0;
        let mut index = 0;
        let mut sentence_count = 0;

        for sentence in &sentences {
            if sentence_count >= max_sentences && !current_chunk.is_empty() {
                chunks.push(DocumentChunk {
                    id: format!("chunk-{}", index),
                    content: current_chunk.clone(),
                    index,
                    start_offset: start,
                    end_offset: start + current_chunk.len(),
                    metadata: std::collections::HashMap::new(),
                });
                index += 1;
                current_chunk.clear();
                start += current_chunk.len();
                sentence_count = 0;
            }
            current_chunk.push_str(sentence);
            current_chunk.push('.');
            sentence_count += 1;
        }

        if !current_chunk.is_empty() {
            chunks.push(DocumentChunk {
                id: format!("chunk-{}", index),
                content: current_chunk,
                index,
                start_offset: start,
                end_offset: content.len(),
                metadata: std::collections::HashMap::new(),
            });
        }

        chunks
    }

    fn smart_split(&self, content: &str) -> Vec<DocumentChunk> {
        let lines_count = content.lines().count();
        let chars_count = content.len();

        if chars_count < 500 {
            return self.split_by_size(content, 500);
        }

        if content.lines().any(|l| l.starts_with('#')) {
            return self.split_by_headers(content);
        }

        if lines_count > 50 {
            return self.split_by_paragraphs(content);
        }

        self.split_by_size(content, 500)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_by_size() {
        let splitter = DocumentSplitter::new(SplitStrategy::BySize { max_chars: 10 });
        let result = splitter.split("doc1", "Hello World Test");
        assert!(result.chunks.len() > 1);
        assert_eq!(result.original_id, "doc1");
    }

    #[test]
    fn test_split_by_headers() {
        let splitter = DocumentSplitter::new(SplitStrategy::ByHeaders);
        let content = "# Header 1\nContent 1\n# Header 2\nContent 2";
        let result = splitter.split("doc1", content);
        assert_eq!(result.chunks.len(), 2);
    }

    #[test]
    fn test_split_by_paragraphs() {
        let splitter = DocumentSplitter::new(SplitStrategy::ByParagraphs);
        let content = "Paragraph 1\n\nParagraph 2\n\nParagraph 3";
        let result = splitter.split("doc1", content);
        assert_eq!(result.chunks.len(), 3);
    }

    #[test]
    fn test_split_by_sentences() {
        let splitter = DocumentSplitter::new(SplitStrategy::BySentences { max_sentences: 2 });
        let content = "First sentence. Second sentence. Third sentence. Fourth sentence.";
        let result = splitter.split("doc1", content);
        assert!(result.chunks.len() >= 2);
    }

    #[test]
    fn test_smart_split_small() {
        let splitter = DocumentSplitter::new(SplitStrategy::Smart);
        let result = splitter.split("doc1", "Small content");
        assert_eq!(result.chunks.len(), 1);
    }
}
pub mod merger;
pub mod splitter;
pub mod archiver;
pub mod dedup;
pub mod quality;

pub use merger::{DocumentMerger, MergeResult, MergeStrategy};
pub use splitter::{DocumentSplitter, SplitResult, SplitStrategy};
pub use archiver::{DocumentArchiver, ArchiveResult, ArchiveReason};
pub use dedup::{DuplicateDetector, DuplicatePair, SimilarityMethod};
pub use quality::{QualityScorer, QualityReport, QualityMetric};

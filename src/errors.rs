use std::fmt;

/// Validation errors when parsing contents of a file.
#[derive(Debug)]
pub enum CitationError {
    /// Two or more possible matches to a single citation. Requires disambiguation through unique key rather than inline citation style.
    AmbiguousMatch(String),

    /// Citations that did not find a match in the source `.bib` bibliography.
    UnmatchedCitations(Vec<String>),
}

impl fmt::Display for CitationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CitationError::AmbiguousMatch(details) => {
                write!(f, "Ambiguous citation match:\n{}", details)
            }
            CitationError::UnmatchedCitations(citations) => {
                write!(f, "Citations not found in the library: {:?}", citations)
            }
        }
    }
}

impl std::error::Error for CitationError {}

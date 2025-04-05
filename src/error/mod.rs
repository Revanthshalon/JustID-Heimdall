#[derive(Debug)]
#[allow(unused)]
pub enum HeimdallError {
    NilSubjectError,
    MalformedInput,
    Database(sqlx::Error),
}

impl std::fmt::Display for HeimdallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeimdallError::NilSubjectError => writeln!(f, "Subject missing"),
            HeimdallError::MalformedInput => writeln!(f, "Malformed Input"),
            HeimdallError::Database(e) => writeln!(f, "Database Error: {e}"),
        }
    }
}

impl std::error::Error for HeimdallError {}

pub type HeimdallResult<T> = Result<T, HeimdallError>;

impl From<sqlx::Error> for HeimdallError {
    fn from(value: sqlx::Error) -> Self {
        HeimdallError::Database(value)
    }
}

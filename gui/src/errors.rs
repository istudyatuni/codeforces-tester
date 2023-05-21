use std::{
    collections::{hash_map::Iter, HashMap},
    path::PathBuf,
};

const BUG_STR: &str = "THIS IS A BUG!";

#[derive(Debug, thiserror::Error, Hash, PartialEq, Eq)]
/// Kind of error for using as key in HashMap
pub(crate) enum ErrorKind {
    #[error("cannot select config")]
    CannotSelectConfig,
    #[error("cannot select path for saving config")]
    CannotSelectPathForSavingConfig,
    #[error("cannot read config")]
    CannotReadConfig,
    #[error("cannot parse config")]
    CannotParseConfig,
    #[error("cannot save config")]
    CannotSaveConfig,
    #[error("cannot open config in editor")]
    CannotOpenConfigInEditor,
    #[error("{} does not exists", .0.display())]
    PathNotExists(PathBuf),

    #[error("self.config is empty when saving config. {BUG_STR}")]
    BugConfigEmptyWhenSavingConfig,
    #[error("self.config_path is empty when saving config. {BUG_STR}")]
    BugConfigPathEmptyWhenSavingConfig,
}

#[derive(Debug, thiserror::Error)]
/// Contains error details
pub(crate) enum Error {
    #[error("")]
    CannotSelectConfig,
    #[error("")]
    CannotSelectPathForSavingConfig,
    #[error("{0}")]
    CannotReadConfig(String),
    #[error("{0}")]
    CannotParseConfig(String, PathBuf),
    #[error("{0}")]
    CannotSaveConfig(String),
    #[error("{0}")]
    CannotOpenConfigInEditor(String),
    #[error("{0}")]
    PathNotExists(String, PathBuf),

    // instead of unreachable!()
    #[error("")]
    BugConfigEmptyWhenSavingConfig,
    #[error("")]
    BugConfigPathEmptyWhenSavingConfig,
}

impl Error {
    pub(crate) fn kind(&self) -> ErrorKind {
        match self {
            Self::CannotSelectConfig => ErrorKind::CannotSelectConfig,
            Self::CannotSelectPathForSavingConfig => ErrorKind::CannotSelectPathForSavingConfig,
            Self::CannotReadConfig(_) => ErrorKind::CannotReadConfig,
            Self::CannotParseConfig(_, _) => ErrorKind::CannotParseConfig,
            Self::CannotSaveConfig(_) => ErrorKind::CannotSaveConfig,
            Self::CannotOpenConfigInEditor(_) => ErrorKind::CannotOpenConfigInEditor,
            Self::PathNotExists(_, path) => ErrorKind::PathNotExists(path.clone()),
            Self::BugConfigEmptyWhenSavingConfig => ErrorKind::BugConfigEmptyWhenSavingConfig,
            Self::BugConfigPathEmptyWhenSavingConfig => {
                ErrorKind::BugConfigPathEmptyWhenSavingConfig
            }
        }
    }
}

#[derive(Debug, Default)]
/// Map for storing errors with ability to delete some kinds of errors
pub(crate) struct ErrorsMap {
    map: HashMap<ErrorKind, Error>,
}

impl ErrorsMap {
    pub(crate) fn add(&mut self, e: Error) {
        self.map.insert(e.kind(), e);
    }
    pub(crate) fn delete(&mut self, kind: ErrorKind) {
        self.map.remove(&kind);
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
    pub(crate) fn clear(&mut self) {
        self.map.clear()
    }
}

impl<'a> IntoIterator for &'a ErrorsMap {
    type Item = (&'a ErrorKind, &'a Error);
    type IntoIter = Iter<'a, ErrorKind, Error>;

    fn into_iter(self) -> Iter<'a, ErrorKind, Error> {
        self.map.iter()
    }
}

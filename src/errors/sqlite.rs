use std::{fmt::Display, sync::PoisonError};

use crate::models::sqlite::SqliteData;

#[derive(Debug)]
pub enum Error {
    NoConnection,
    TableNotFound,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoConnection => write!(f, "No connection to database"),
            Self::TableNotFound => write!(f, "Table not found"),
        }
    }
}

impl From<Error> for &str {
    fn from(val: Error) -> Self {
        match val {
            Error::NoConnection => "No connection to database",
            Error::TableNotFound => "Table not found",
        }
    }
}

impl std::error::Error for Error {}

impl From<r2d2_sqlite::SqliteConnectionManager> for Error {
    fn from(_: r2d2_sqlite::SqliteConnectionManager) -> Self {
        Self::NoConnection
    }
}

impl From<Error> for r2d2_sqlite::SqliteConnectionManager {
    fn from(_val: Error) -> Self {
        Self::file("errors.sqlite.db")
    }
}

impl From<r2d2::Error> for Error {
    fn from(_: r2d2::Error) -> Self {
        Self::NoConnection
    }
}

impl From<PoisonError<std::sync::MutexGuard<'_, Vec<SqliteData>>>> for Error {
    fn from(_: PoisonError<std::sync::MutexGuard<'_, Vec<SqliteData>>>) -> Self {
        Self::NoConnection
    }
}

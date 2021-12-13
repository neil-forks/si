use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimestampError {}

pub type TimestampResult<T> = Result<T, TimestampError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct Timestamp {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
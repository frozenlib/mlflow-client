use std::time::SystemTime;

use derive_ex::Ex;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// <https://mlflow.org/docs/latest/rest-api.html#fileinfo>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct FileInfo {
    pub path: String,
    pub is_dir: bool,
    pub file_size: Option<i64>,
}

/// <https://mlflow.org/docs/latest/rest-api.html#run>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Run {
    pub info: RunInfo,
    pub data: RunData,
    pub inputs: RunInputs,
}

/// <https://mlflow.org/docs/latest/rest-api.html#runinfo>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RunInfo {
    pub run_id: String,
    pub run_name: String,
    pub experiment_id: String,
    pub status: RunStatus,
    pub start_time: Timestamp,
    pub end_time: Option<Timestamp>,
    pub artifact_uri: String,
    pub lifecycle_stage: String,
}

/// <https://mlflow.org/docs/latest/rest-api.html#rundata>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RunData {
    #[serde(default)]
    pub metrics: Vec<Metric>,
    #[serde(default)]
    pub params: Vec<Param>,
    #[serde(default)]
    pub tags: Vec<RunTag>,
}

/// <https://mlflow.org/docs/latest/rest-api.html#metric>
#[derive(Serialize, Deserialize, Debug, Clone, Ex)]
#[derive_ex(Eq, PartialEq, Ord, PartialOrd)]
pub struct Metric {
    pub key: String,
    #[ord(key = OrderedFloat($))]
    pub value: f64,
    pub timestamp: Timestamp,
    pub step: Option<i64>,
}

/// <https://mlflow.org/docs/latest/rest-api.html#param>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Param {
    pub key: String,
    pub value: String,
}

/// <https://mlflow.org/docs/latest/rest-api.html#runinputs>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RunInputs {
    #[serde(default)]
    pub dataset_inputs: Vec<DatasetInput>,
}

/// <https://mlflow.org/docs/latest/rest-api.html#datasetinput>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DatasetInput {
    pub tags: Vec<InputTag>,
    pub dataset: Dataset,
}

/// <https://mlflow.org/docs/latest/rest-api.html#inputtag>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct InputTag {
    pub key: String,
    pub value: String,
}

/// <https://mlflow.org/docs/latest/rest-api.html#dataset>
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Dataset {
    pub name: String,
    pub digest: String,
    pub source_type: String,
    pub source: String,
    pub schema: Option<String>,
    pub profile: Option<String>,
}

/// <https://mlflow.org/docs/latest/rest-api.html#runstatus>
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunStatus {
    Running,
    Scheduled,
    Finished,
    Failed,
    Killed,
}

/// <https://mlflow.org/docs/latest/rest-api.html#experiment>
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Experiment {
    pub experiment_id: String,
    pub name: String,
    pub artifact_location: String,
    pub lifecycle_stage: String,
    pub last_update_time: Timestamp,
    pub creation_time: Timestamp,
    #[serde(default)]
    pub tags: Vec<ExperimentTag>,
}

/// <https://mlflow.org/docs/latest/rest-api.html#viewtype>
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ViewType {
    #[default]
    ActiveOnly,
    DeletedOnly,
    All,
}

/// <https://mlflow.org/docs/latest/rest-api.html#experimenttag>
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ExperimentTag {
    pub key: String,
    pub value: String,
}

/// <https://mlflow.org/docs/latest/rest-api.html#runtag>
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RunTag {
    pub key: String,
    pub value: String,
}

/// Unix timestamp in milliseconds.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[serde(transparent)]
pub struct Timestamp(pub i64);

impl Timestamp {
    pub fn now() -> Self {
        SystemTime::now().try_into().unwrap()
    }
}

impl From<i64> for Timestamp {
    fn from(i: i64) -> Self {
        Timestamp(i)
    }
}
impl From<Timestamp> for i64 {
    fn from(ts: Timestamp) -> i64 {
        ts.0
    }
}

impl TryFrom<SystemTime> for Timestamp {
    type Error = TimestampError;
    fn try_from(value: SystemTime) -> Result<Self, Self::Error> {
        Ok(Timestamp(if value < SystemTime::UNIX_EPOCH {
            let millis: i128 = SystemTime::UNIX_EPOCH
                .duration_since(value)?
                .as_millis()
                .try_into()?;
            (-millis).try_into()?
        } else {
            value
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis()
                .try_into()?
        }))
    }
}
impl From<Timestamp> for SystemTime {
    fn from(value: Timestamp) -> SystemTime {
        if value.0 < 0 {
            SystemTime::UNIX_EPOCH - std::time::Duration::from_millis(-(value.0 as i128) as u64)
        } else {
            SystemTime::UNIX_EPOCH + std::time::Duration::from_millis(value.0 as u64)
        }
    }
}

#[derive(Debug, Error)]
pub enum TimestampError {
    #[error("SystemTimeError: {0}")]
    SystemTimeError(#[from] std::time::SystemTimeError),
    #[error("TryFromIntError: {0}")]
    TryFromIntError(#[from] std::num::TryFromIntError),
}

/// <https://mlflow.org/docs/latest/rest-api.html#request-structure>
#[derive(Serialize, Debug, Clone, Copy, Default)]
pub struct CreateExperimentOptions<'a> {
    pub artifact_location: Option<&'a str>,
    pub tags: &'a [ExperimentTag],
}

/// <https://mlflow.org/docs/latest/rest-api.html#mlflowsearchexperiments>
#[derive(Serialize, Debug, Clone, Copy, Ex)]
#[derive_ex(Default)]
pub struct SearchExperimentsOptions<'a> {
    pub filter: &'a str,
    pub order_by: &'a [&'a str],
    pub view_type: ViewType,
}

/// <https://mlflow.org/docs/latest/rest-api.html#mlflowcreaterun>
#[derive(Serialize, Debug, Clone, Copy, Default)]
pub struct CreateRunOptions<'a> {
    pub start_time: Option<Timestamp>,
    pub tags: &'a [RunTag],
}

/// <https://mlflow.org/docs/latest/rest-api.html#mlflowsearchruns>
#[derive(Serialize, Debug, Clone, Copy, Ex)]
#[derive_ex(Default)]
pub struct SearchRunsOptions<'a> {
    pub filter: &'a str,
    pub run_view_type: ViewType,
    pub order_by: &'a [&'a str],
}

/// <https://mlflow.org/docs/latest/rest-api.html#mlflowupdaterun>
#[derive(Serialize, Debug, Clone, Copy, Default)]
pub struct UpdateRunOptions<'a> {
    pub status: Option<RunStatus>,
    pub end_time: Option<Timestamp>,
    pub run_name: Option<&'a str>,
}

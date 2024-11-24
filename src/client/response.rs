use serde::{Deserialize, Serialize};

use crate::data::{Experiment, FileInfo, Metric, Run, RunInfo};

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateExperimentResponse {
    pub experiment_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchExperimentsResponse {
    #[serde(default)]
    pub experiments: Vec<Experiment>,
    pub next_page_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetExperimentResponse {
    pub experiment: Experiment,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetRunResponse {
    pub run: Run,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetMetricHistoryResponse {
    pub metrics: Vec<Metric>,
    pub next_page_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchRunsResponse {
    #[serde(default)]
    pub runs: Vec<Run>,
    pub next_page_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListArtifactsResponse {
    pub root_uri: String,
    pub files: Vec<FileInfo>,
    pub page_token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateRunResponse {
    pub run_info: RunInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse {
    pub error_code: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnitResponse {}

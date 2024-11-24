use std::sync::Arc;

use reqwest::{
    blocking::{Client, Response},
    Url,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

use crate::{
    data::{
        CreateExperimentOptions, CreateRunOptions, DatasetInput, Metric, Param, RunTag,
        SearchExperimentsOptions, SearchRunsOptions, Timestamp, UpdateRunOptions,
    },
    Error, Result,
};

pub mod response;

use response::*;

#[derive(Debug, Clone)]
pub struct MlflowClient {
    uri: Arc<Url>,
}

impl MlflowClient {
    pub fn new(uri: &str) -> Result<MlflowClient> {
        Ok(MlflowClient {
            uri: Arc::new(Url::parse(uri)?),
        })
    }
    /// <https://mlflow.org/docs/latest/rest-api.html#create-experiment>
    pub fn create_experiment(
        &self,
        name: &str,
        options: CreateExperimentOptions,
    ) -> Result<CreateExperimentResponse> {
        let body = build_body(json!({ "name": name }), options)?;
        self.post("experiments/create", body)
    }

    pub const SEARCH_EXPERIMENTS_MAX_RESULTS_SUPPORTED: i64 = 1000;

    /// <https://mlflow.org/docs/latest/rest-api.html#search-experiments>
    pub fn search_experiments(
        &self,
        options: SearchExperimentsOptions,
        max_results: i64,
        page_token: Option<&str>,
    ) -> Result<SearchExperimentsResponse> {
        let body = build_body(
            json!({ "max_results": max_results, "page_token": page_token}),
            options,
        )?;
        self.post("experiments/search", body)
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#get-experiment>
    pub fn get_experiment(&self, experiment_id: &str) -> Result<GetExperimentResponse> {
        self.get("experiments/get", &[("experiment_id", experiment_id)])
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#get-experiment-by-name>
    pub fn get_experiment_by_name(&self, experiment_name: &str) -> Result<GetExperimentResponse> {
        self.get(
            "experiments/get-by-name",
            &[("experiment_name", experiment_name)],
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#delete-experiment>
    pub fn delete_experiment(&self, experiment_id: &str) -> Result<UnitResponse> {
        self.post(
            "experiments/delete",
            json!({ "experiment_id": experiment_id }),
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#restore-experiment>
    pub fn restore_experiment(&self, experiment_id: &str) -> Result<UnitResponse> {
        self.post(
            "experiments/restore",
            json!({ "experiment_id": experiment_id }),
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#update-experiment>
    pub fn update_experiment(&self, experiment_id: &str, new_name: &str) -> Result<UnitResponse> {
        self.post(
            "experiments/update",
            json!({
                "experiment_id": experiment_id,
                "new_name": new_name,
            }),
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#create-run>
    pub fn create_run(
        &self,
        experiment_id: &str,
        run_name: &str,
        options: CreateRunOptions,
    ) -> Result<GetRunResponse> {
        let body = build_body(
            json!({ "experiment_id": experiment_id, "run_name": run_name }),
            options,
        )?;
        self.post("runs/create", body)
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#delete-run>
    pub fn delete_run(&self, run_id: &str) -> Result<UnitResponse> {
        self.post("runs/delete", json!({ "run_id": run_id }))
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#restore-run>
    pub fn restore_run(&self, run_id: &str) -> Result<UnitResponse> {
        self.post("runs/restore", json!({ "run_id": run_id }))
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#get-run>
    pub fn get_run(&self, run_id: &str) -> Result<GetRunResponse> {
        self.get("runs/get", &[("run_id", run_id)])
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#log-metric>
    pub fn log_metric(
        &self,
        run_id: &str,
        key: &str,
        value: f64,
        timestamp: Timestamp,
        step: Option<i64>,
    ) -> Result<UnitResponse> {
        self.post(
            "runs/log-metric",
            json!({
                "run_id": run_id,
                "key": key,
                "value": value,
                "timestamp": timestamp,
                "step": step,
            }),
        )
    }

    pub const LOG_BATCH_MAX_TOTAL: usize = 1000;
    pub const LOG_BATCH_MAX_METRICS: usize = 1000;
    pub const LOG_BATCH_MAX_PARAMS: usize = 100;
    pub const LOG_BATCH_MAX_TAGS: usize = 100;

    /// <https://mlflow.org/docs/latest/rest-api.html#log-batch>
    pub fn log_batch(
        &self,
        run_id: &str,
        metrics: &[Metric],
        params: &[Param],
        tags: &[RunTag],
    ) -> Result<UnitResponse> {
        self.post(
            "runs/log-batch",
            json!({
                "run_id": run_id,
                "metrics": metrics,
                "params": params,
                "tags": tags,
            }),
        )
    }

    // /// <https://mlflow.org/docs/latest/rest-api.html#log-model>
    // pub fn log_model(&self, run_id: &str, model_json: &str) -> Result<UnitResponse> {
    //     self.post(
    //         "runs/log-model",
    //         json!({
    //             "run_id": run_id,
    //             "model_json": model_json,
    //         }),
    //     )
    // }

    /// <https://mlflow.org/docs/latest/rest-api.html#log-inputs>
    pub fn log_inputs(&self, run_id: &str, datasets: &[DatasetInput]) -> Result<UnitResponse> {
        self.post(
            "runs/log-inputs",
            json!({
                "run_id": run_id,
                "datasets": datasets,
            }),
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#set-experiment-tag>
    pub fn set_experiment_tag(
        &self,
        experiment_id: &str,
        key: &str,
        value: &str,
    ) -> Result<UnitResponse> {
        self.post(
            "experiments/set-experiment-tag",
            json!({
                "experiment_id": experiment_id,
                "key": key,
                "value": value,
            }),
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#set-tag>
    pub fn set_tag(&self, run_id: &str, key: &str, value: &str) -> Result<UnitResponse> {
        self.post(
            "runs/set-tag",
            json!({
                "run_id": run_id,
                "key": key,
                "value": value,
            }),
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#delete-tag>
    pub fn delete_tag(&self, run_id: &str, key: &str) -> Result<UnitResponse> {
        self.post(
            "runs/delete-tag",
            json!({
                "run_id": run_id,
                "key": key,
            }),
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#log-param>
    pub fn log_param(&self, run_id: &str, key: &str, value: &str) -> Result<UnitResponse> {
        self.post(
            "runs/log-parameter",
            json!({
                "run_id": run_id,
                "key": key,
                "value": value,
            }),
        )
    }

    /// <https://mlflow.org/docs/latest/rest-api.html#get-metric-history>
    pub fn get_metric_history(
        &self,
        run_id: &str,
        metric_key: &str,
        max_results: i32,
        page_token: Option<&str>,
    ) -> Result<GetMetricHistoryResponse> {
        self.get(
            "metrics/get-history",
            &[
                ("run_id", run_id),
                ("metric_key", metric_key),
                ("max_results", &max_results.to_string()),
                ("page_token", page_token.unwrap_or("")),
            ],
        )
    }

    pub const SEARCH_RUNS_MAX_RESULTS_SUPPORTED: i32 = 50000;

    /// <https://mlflow.org/docs/latest/rest-api.html#search-runs>
    pub fn search_runs(
        &self,
        experiment_ids: &[&str],
        options: SearchRunsOptions,
        max_results: i32,
        page_token: Option<&str>,
    ) -> Result<SearchRunsResponse> {
        let body = build_body(
            json!({ "experiment_ids": experiment_ids, "max_results" : max_results, "page_token": page_token }),
            options,
        )?;
        self.post("runs/search", body)
    }

    // /// <https://mlflow.org/docs/latest/rest-api.html#list-artifacts>
    // pub fn list_artifacts(
    //     &self,
    //     run_id: &str,
    //     path: &str,
    //     page_token: Option<&str>,
    // ) -> Result<ListArtifactsResponse> {
    //     self.get(
    //         "artifacts/list",
    //         &[
    //             ("run_id", run_id),
    //             ("path", path),
    //             ("page_token", page_token.unwrap_or("")),
    //         ],
    //     )
    // }

    /// <https://mlflow.org/docs/latest/rest-api.html#update-run>
    pub fn update_run(&self, run_id: &str, options: UpdateRunOptions) -> Result<UpdateRunResponse> {
        let body = build_body(json!({ "run_id":run_id }), options)?;
        self.post("runs/update", body)
    }

    fn post<T: DeserializeOwned>(&self, path: &str, body: impl Serialize) -> Result<T> {
        to_result(Client::new().post(self.url(path)?).json(&body).send()?)
    }
    fn get<T: DeserializeOwned>(&self, path: &str, query: &[(&str, &str)]) -> Result<T> {
        to_result(Client::new().get(self.url(path)?).query(query).send()?)
    }

    fn url(&self, path: &str) -> Result<Url> {
        Ok(self.uri.join("/api/2.0/mlflow/")?.join(path)?)
    }
}
impl Default for MlflowClient {
    fn default() -> Self {
        MlflowClient::new("http://localhost:5000").unwrap()
    }
}

fn to_result<T: DeserializeOwned>(r: Response) -> Result<T> {
    if r.status().is_success() {
        Ok(r.json()?)
    } else {
        let e: ErrorResponse = r.json()?;
        Err(Error::ApiError {
            error_code: e.error_code,
            message: e.message,
        })
    }
}
fn build_body(json: Value, options: impl Serialize) -> Result<Value> {
    let Value::Object(mut l) = json else {
        panic!("l: expected object");
    };
    let r = serde_json::to_value(options)?;
    let Value::Object(r) = r else {
        panic!("r: expected object");
    };
    for (k, v) in r {
        l.insert(k, v);
    }
    Ok(Value::Object(l))
}

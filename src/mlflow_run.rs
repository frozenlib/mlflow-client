use serde::Serialize;

use crate::client::MlflowClient;
use crate::data::{Metric, Param, Run, RunTag, Timestamp, UpdateRunOptions};
use crate::utils::build_params;
use crate::{MlflowRunWriter, Result};

#[derive(Debug, Clone)]
pub struct MlflowRun {
    client: MlflowClient,
    data: Run,
}
impl MlflowRun {
    pub(crate) fn new(client: &MlflowClient, data: Run) -> MlflowRun {
        MlflowRun {
            client: client.clone(),
            data,
        }
    }
    pub fn id(&self) -> &str {
        &self.data.info.run_id
    }
    pub fn name(&self) -> &str {
        &self.data.info.run_name
    }
    pub fn data(&self) -> &Run {
        &self.data
    }

    pub fn reload(&self) -> Result<Self> {
        Ok(MlflowRun::new(
            &self.client,
            self.client.get_run(self.id())?.run,
        ))
    }
    pub fn update(&self, options: UpdateRunOptions) -> Result<()> {
        self.client.update_run(self.id(), options)?;
        Ok(())
    }

    pub fn delete(&self) -> Result<()> {
        self.client.delete_run(self.id())?;
        Ok(())
    }
    pub fn restore(&self) -> Result<()> {
        self.client.restore_run(self.id())?;
        Ok(())
    }
    pub fn set_tag(&self, key: &str, value: &str) -> Result<()> {
        self.client.set_tag(self.id(), key, value)?;
        Ok(())
    }
    pub fn delete_tag(&self, key: &str) -> Result<()> {
        self.client.delete_tag(self.id(), key)?;
        Ok(())
    }

    pub fn log_param(&self, key: &str, value: &str) -> Result<()> {
        self.client.log_param(self.id(), key, value)?;
        Ok(())
    }
    pub fn log_params(&self, key: &str, values: impl Serialize) -> Result<()> {
        let values = serde_json::to_value(values)?;
        let mut params = Vec::new();
        build_params(key, &values, &mut params)?;
        self.log_batch(&[], &params, &[])?;
        Ok(())
    }
    pub fn log_metric(
        &self,
        key: &str,
        value: f64,
        timestamp: impl Into<Timestamp>,
        step: Option<i64>,
    ) -> Result<()> {
        self.client
            .log_metric(self.id(), key, value, timestamp.into(), step)?;
        Ok(())
    }
    pub fn log_metrics(
        &mut self,
        metrics: &[(impl AsRef<str>, f64)],
        step: Option<i64>,
    ) -> Result<()> {
        let timestamp = Timestamp::now();
        let metrics = metrics
            .iter()
            .map(|(key, value)| Metric {
                key: key.as_ref().to_string(),
                value: *value,
                timestamp,
                step,
            })
            .collect::<Vec<_>>();
        self.log_batch(&metrics, &[], &[])
    }

    pub fn log_batch(&self, metrics: &[Metric], params: &[Param], tags: &[RunTag]) -> Result<()> {
        let len_sum = metrics.len() + params.len() + tags.len();
        if len_sum == 0 {
            return Ok(());
        }
        if len_sum <= MlflowClient::LOG_BATCH_MAX_TOTAL
            && metrics.len() <= MlflowClient::LOG_BATCH_MAX_METRICS
            && params.len() <= MlflowClient::LOG_BATCH_MAX_PARAMS
            && tags.len() <= MlflowClient::LOG_BATCH_MAX_TAGS
        {
            self.client.log_batch(self.id(), metrics, params, tags)?;
            return Ok(());
        }
        let mut end = 0;
        while end < metrics.len() {
            let start = end;
            end = (end + MlflowClient::LOG_BATCH_MAX_METRICS).min(metrics.len());
            self.client
                .log_batch(self.id(), &metrics[start..end], &[], &[])?;
        }
        let mut end = 0;
        while end < params.len() {
            let start = end;
            end = (end + MlflowClient::LOG_BATCH_MAX_PARAMS).min(params.len());
            self.client
                .log_batch(self.id(), &[], &params[start..end], &[])?;
        }
        let mut end = 0;
        while end < tags.len() {
            let start = end;
            end = (end + MlflowClient::LOG_BATCH_MAX_TAGS).min(tags.len());
            self.client
                .log_batch(self.id(), &[], &[], &tags[start..end])?;
        }
        Ok(())
    }

    pub fn metric_history(&self, key: &str) -> Result<Vec<Metric>> {
        let mut results = Vec::new();
        let mut page_token = None;
        loop {
            let response =
                self.client
                    .get_metric_history(self.id(), key, 1000, page_token.as_deref())?;
            results.extend(response.metrics);
            page_token = response.next_page_token;
            if page_token.is_none() {
                break;
            }
        }
        Ok(results)
    }
    pub(crate) fn writer(&self) -> MlflowRunWriter {
        MlflowRunWriter::new(self.clone())
    }
}

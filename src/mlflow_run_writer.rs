use std::{
    mem::take,
    sync::{Arc, Mutex},
    thread::{spawn, JoinHandle},
};

use serde::Serialize;

use crate::{
    data::{Metric, RunStatus, Timestamp, UpdateRunOptions},
    Error, MlflowRun, Result,
};

struct Data {
    metrics: Vec<Metric>,
    error: Option<Error>,
    status: RunStatus,
    end_time: Option<Timestamp>,
    task: Option<JoinHandle<()>>,
}
impl Data {
    fn take_error(&mut self) -> Result<()> {
        if let Some(task) = self.task.as_ref() {
            if task.is_finished() {
                if let Some(task) = self.task.take() {
                    if task.join().is_err() {
                        return Err(Error::TaskJoinError);
                    }
                }
            }
        }
        if let Some(e) = self.error.take() {
            Err(e)
        } else {
            Ok(())
        }
    }
    fn push_error(&mut self, e: Option<Error>) {
        if self.error.is_none() {
            self.error = e;
        }
    }
}

/// Writer for outputting logs to [Run](https://mlflow.org/docs/latest/tracking.html#runs).
///
/// This differs from [`MlflowRun`] in the following ways:
/// - Methods other than [`finish`](Self::finish) may output logs asynchronously for performance reasons.
///   When output is asynchronous, any errors that occur will be returned in subsequent method calls.
/// - If an instance is dropped without calling [`finish`](Self::finish), the Run's status will be set to Failed.
/// - Log timestamps will be set to the time when the method is called
///
/// To obtain a `MlflowRunWriter`, use [`MlflowExperiment::start_run`] or [`MlflowExperiment::start_run_with`].
///
/// [`MlflowExperiment::start_run`]: crate::MlflowExperiment::start_run
/// [`MlflowExperiment::start_run_with`]: crate::MlflowExperiment::start_run_with
pub struct MlflowRunWriter {
    run: MlflowRun,
    is_end: bool,
    data: Arc<Mutex<Data>>,
}

impl MlflowRunWriter {
    pub(crate) fn new(run: MlflowRun) -> Self {
        Self {
            run,
            is_end: false,
            data: Arc::new(Mutex::new(Data {
                metrics: Vec::new(),
                error: None,
                status: RunStatus::Running,
                end_time: None,
                task: None,
            })),
        }
    }
    pub fn run(&self) -> &MlflowRun {
        &self.run
    }

    /// Logs a single parameter.
    pub fn log_param(&mut self, key: &str, value: &str) -> Result<()> {
        self.run.log_param(key, value)
    }

    /// Logs multiple parameters.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mlflow = mlflow_client::Mlflow::new("http://localhost:5000")?;
    /// let experiment = mlflow.create_experiment_if_not_exists("experiment_name", Default::default())?;
    /// let run = experiment.start_run("run_name")?;
    ///
    /// #[derive(serde::Serialize)]
    /// struct HyperParams {
    ///    param_a: f64,
    ///    param_b: f64,
    /// }
    /// let params = HyperParams {
    ///   param_a: 1.0,
    ///   param_b: 2.0,
    /// };
    /// run.log_params("prefix", params)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn log_params(&mut self, key: &str, values: impl Serialize) -> Result<()> {
        self.run.log_params(key, values)
    }
    pub fn log_metric(&mut self, key: &str, value: f64, step: Option<i64>) -> Result<()> {
        let mut d = self.data.lock().unwrap();
        d.metrics.push(Metric {
            key: key.to_string(),
            value,
            timestamp: Timestamp::now(),
            step,
        });
        self.spawn_task(&mut d);
        d.take_error()?;
        Ok(())
    }
    pub fn log_metrics(
        &mut self,
        metrics: &[(impl AsRef<str>, f64)],
        step: Option<i64>,
    ) -> Result<()> {
        let timestamp = Timestamp::now();
        let mut d = self.data.lock().unwrap();
        for (key, value) in metrics {
            d.metrics.push(Metric {
                key: key.as_ref().to_string(),
                value: *value,
                timestamp,
                step,
            });
        }
        self.spawn_task(&mut d);
        d.take_error()?;
        Ok(())
    }

    /// Finish the run with the status [`Finished`](RunStatus::Finished).
    ///
    /// If this method is not called and the `MlflowRunWriter` is dropped, the status will be [`Failed`](RunStatus::Failed).
    #[doc(alias = "end_run")]
    pub fn finish(mut self) -> Result<()> {
        self.end(RunStatus::Finished)
    }
    fn end(&mut self, status: RunStatus) -> Result<()> {
        self.is_end = true;

        let mut d = self.data.lock().unwrap();
        d.status = status;
        d.end_time = Some(Timestamp::now());
        self.spawn_task(&mut d);
        let task = d.task.take();
        drop(d);
        if let Some(task) = task {
            if task.join().is_err() {
                return Err(Error::TaskJoinError);
            }
        }
        let mut d = self.data.lock().unwrap();
        d.take_error()?;
        Ok(())
    }

    fn spawn_task(&self, d: &mut Data) {
        if d.task.is_none() {
            let run = self.run.clone();
            let data = self.data.clone();
            d.task = Some(spawn(move || run_task(run, data)));
        }
    }
}
impl Drop for MlflowRunWriter {
    fn drop(&mut self) {
        if !self.is_end {
            let _ = self.end(RunStatus::Failed);
        }
    }
}

fn run_task(run: MlflowRun, data: Arc<Mutex<Data>>) {
    let mut err = None;
    loop {
        let mut d = data.lock().unwrap();
        if d.error.is_none() && !d.metrics.is_empty() {
            let metrics = take(&mut d.metrics);
            drop(d);
            if let Err(e) = run.log_batch(&metrics, &[], &[]) {
                err = err.or(Some(e));
            }
            continue;
        }
        if d.status != RunStatus::Running {
            let options = UpdateRunOptions {
                status: Some(d.status),
                end_time: d.end_time,
                ..Default::default()
            };
            drop(d);
            if let Err(e) = run.update(options) {
                err = err.or(Some(e));
            }
            d = data.lock().unwrap();
        }
        d.push_error(err);
        d.task.take();
        break;
    }
}

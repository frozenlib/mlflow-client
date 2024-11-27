use crate::client::MlflowClient;
use crate::data::{CreateRunOptions, Experiment, SearchRunsOptions, Timestamp};
use crate::utils::none_if_not_exist;
use crate::{MlflowRun, MlflowRunWriter, Result};

/// Represents a [Experiment](https://mlflow.org/docs/latest/tracking.html#experiments).
#[derive(Debug, Clone)]
pub struct MlflowExperiment {
    client: MlflowClient,
    data: Experiment,
}

impl MlflowExperiment {
    pub(crate) fn new(client: &MlflowClient, data: Experiment) -> MlflowExperiment {
        MlflowExperiment {
            client: client.clone(),
            data,
        }
    }
    pub fn id(&self) -> &str {
        &self.data.experiment_id
    }
    pub fn name(&self) -> &str {
        &self.data.name
    }
    pub fn data(&self) -> &Experiment {
        &self.data
    }

    pub fn reload(&self) -> Result<Self> {
        Ok(MlflowExperiment::new(
            &self.client,
            self.client.get_experiment(self.id())?.experiment,
        ))
    }

    pub fn delete(&self) -> Result<()> {
        self.client.delete_experiment(self.id())?;
        Ok(())
    }
    pub fn restore(&self) -> Result<()> {
        self.client.restore_experiment(self.id())?;
        Ok(())
    }
    pub fn update(&self, new_name: &str) -> Result<()> {
        self.client.update_experiment(self.id(), new_name)?;
        Ok(())
    }
    pub fn set_tag(&self, key: &str, value: &str) -> Result<()> {
        self.client.set_experiment_tag(self.id(), key, value)?;
        Ok(())
    }

    /// Get all active runs in this experiment.
    pub fn runs(&self) -> Result<Vec<MlflowRun>> {
        self.runs_with(SearchRunsOptions::default())
    }

    /// Get all runs in this experiment that match the specified search options.
    pub fn runs_with(&self, options: SearchRunsOptions) -> Result<Vec<MlflowRun>> {
        let mut results = Vec::new();
        let mut page_token = None;
        loop {
            let response = self.client.search_runs(
                &[self.id()],
                options,
                MlflowClient::SEARCH_RUNS_MAX_RESULTS_SUPPORTED,
                page_token.as_deref(),
            )?;
            for run in response.runs {
                results.push(MlflowRun::new(&self.client, run));
            }
            page_token = response.next_page_token;
            if page_token.is_none() {
                break;
            }
        }
        Ok(results)
    }

    /// Get a run by its ID.
    pub fn run(&self, id: &str) -> Result<Option<MlflowRun>> {
        none_if_not_exist(self.client.get_run(id), |r| {
            Ok(MlflowRun::new(&self.client, r.run))
        })
    }

    /// Create a new Run.
    ///
    /// Use [`start_run`](Self::start_run) instead of `create_run` to log the currently running run.
    /// [`MlflowRunWriter`] contains features suitable for logging the currently running run.
    pub fn create_run(&self, name: &str, options: CreateRunOptions) -> Result<MlflowRun> {
        let r = self.client.create_run(self.id(), name, options)?;
        Ok(MlflowRun::new(&self.client, r.run))
    }

    /// Creates a Run with default settings and returns its [`MlflowRunWriter`].
    pub fn start_run(&self, name: &str) -> Result<MlflowRunWriter> {
        self.start_run_with(name, CreateRunOptions::default())
    }

    /// Creates a Run with the specified options and returns its [`MlflowRunWriter`].
    ///
    /// `options.start_time` is set to the current time if not specified.
    pub fn start_run_with(
        &self,
        name: &str,
        mut options: CreateRunOptions,
    ) -> Result<MlflowRunWriter> {
        if options.start_time.is_none() {
            options.start_time = Some(Timestamp::now());
        }
        Ok(self.create_run(name, options)?.writer())
    }
}

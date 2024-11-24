use crate::client::MlflowClient;
use crate::data::{CreateExperimentOptions, SearchExperimentsOptions};
use crate::utils::none_if_not_exist;
use crate::{MlflowExperiment, Result};

#[derive(Debug, Clone, Default)]
pub struct Mlflow {
    client: MlflowClient,
}
impl Mlflow {
    /// Create a new `Mlflow` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use mlflow_client::Mlflow;
    ///
    /// let _mlflow = Mlflow::new("http://localhost:5000").unwrap();
    /// ```
    pub fn new(uri: &str) -> Result<Mlflow> {
        Ok(Mlflow {
            client: MlflowClient::new(uri)?,
        })
    }

    /// Get all active experiments.
    pub fn experiments(&self) -> Result<Vec<MlflowExperiment>> {
        self.experiments_with(SearchExperimentsOptions::default())
    }
    pub fn experiments_with(
        &self,
        options: SearchExperimentsOptions,
    ) -> Result<Vec<MlflowExperiment>> {
        let mut results = Vec::new();
        let mut page_token = None;
        loop {
            let response = self.client.search_experiments(
                options,
                MlflowClient::SEARCH_EXPERIMENTS_MAX_RESULTS_SUPPORTED,
                page_token.as_deref(),
            )?;
            for experiment in response.experiments {
                results.push(MlflowExperiment::new(&self.client, experiment));
            }
            page_token = response.next_page_token;
            if page_token.is_none() {
                break;
            }
        }
        Ok(results)
    }
    pub fn experiment(&self, id: &str) -> Result<Option<MlflowExperiment>> {
        none_if_not_exist(self.client.get_experiment(id), |r| {
            Ok(MlflowExperiment::new(&self.client, r.experiment))
        })
    }
    pub fn experiment_by_name(&self, name: &str) -> Result<Option<MlflowExperiment>> {
        none_if_not_exist(self.client.get_experiment_by_name(name), |r| {
            Ok(MlflowExperiment::new(&self.client, r.experiment))
        })
    }

    pub fn create_experiment(
        &self,
        name: &str,
        options: CreateExperimentOptions,
    ) -> Result<MlflowExperiment> {
        let r = self.client.create_experiment(name, options)?;
        let r = self.client.get_experiment(&r.experiment_id)?;
        Ok(MlflowExperiment::new(&self.client, r.experiment))
    }

    pub fn create_experiment_if_not_exists(
        &self,
        name: &str,
        options: CreateExperimentOptions,
    ) -> Result<MlflowExperiment> {
        if let Some(experiment) = self.experiment_by_name(name)? {
            return Ok(experiment);
        }
        self.create_experiment(name, options)
    }
}

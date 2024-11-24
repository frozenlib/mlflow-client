use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

mod error;
mod mlflow;
mod mlflow_experiment;
mod mlflow_run;
mod mlflow_run_writer;
mod utils;

pub use error::Error;
pub use mlflow::Mlflow;
pub use mlflow_experiment::MlflowExperiment;
pub use mlflow_run::MlflowRun;
pub use mlflow_run_writer::MlflowRunWriter;

pub mod client;
pub mod data;

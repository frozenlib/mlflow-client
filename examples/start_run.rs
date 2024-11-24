fn main() -> anyhow::Result<()> {
    let mlflow = mlflow_client::Mlflow::new("http://localhost:5000")?;
    let experiment =
        mlflow.create_experiment_if_not_exists("experiment_name", Default::default())?;
    let mut run = experiment.start_run("run_name")?;

    #[derive(serde::Serialize)]
    struct HyperParams {
        param_a: f64,
        param_b: f64,
    }
    let params = HyperParams {
        param_a: 1.0,
        param_b: 2.0,
    };
    run.log_params("", params)?;

    for epoch in 0..100 {
        run.log_metric("loss", 0.5, Some(epoch))?;
    }
    run.finish()?;
    Ok(())
}

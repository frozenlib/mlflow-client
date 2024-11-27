# mlflow-client

[![Crates.io](https://img.shields.io/crates/v/mlflow-client.svg)](https://crates.io/crates/mlflow-client)
[![Docs.rs](https://docs.rs/mlflow-client/badge.svg)](https://docs.rs/mlflow-client/)
[![Actions Status](https://github.com/frozenlib/mlflow-client/workflows/CI/badge.svg)](https://github.com/frozenlib/mlflow-client/actions)

[MLflow](https://mlflow.org/) REST API client for Rust. (Unofficial)

## Supported APIs

- [x] [MLflow Tracking](https://mlflow.org/docs/latest/tracking.html)

## Example

```rust
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
```

## License

This project is dual licensed under Apache-2.0/MIT. See the two LICENSE-\* files for details.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

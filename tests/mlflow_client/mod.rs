use anyhow::Result;
use mlflow_client::{
    client::MlflowClient,
    data::{
        CreateRunOptions, Dataset, DatasetInput, InputTag, Metric, Param, RunTag,
        SearchExperimentsOptions, SearchRunsOptions, UpdateRunOptions, ViewType,
    },
};

use crate::MlflowServer;

#[test]
fn create_experiment() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.get_experiment(&r0.experiment_id)?;
    let e = r1.experiment;
    assert_eq!(e.experiment_id, r0.experiment_id);
    assert_eq!(e.name, "abc");
    Ok(())
}

#[test]
fn search_experiments() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let options = SearchExperimentsOptions {
        order_by: &["creation_time"],
        ..Default::default()
    };
    let max_results = MlflowClient::SEARCH_EXPERIMENTS_MAX_RESULTS_SUPPORTED;
    let page_token = None;
    let r1 = c.search_experiments(options, max_results, page_token)?;

    assert_eq!(r1.experiments.len(), 2);
    assert_eq!(r1.experiments[0].experiment_id, "0");
    assert_eq!(r1.experiments[0].name, "Default");
    assert_eq!(r1.experiments[1].experiment_id, r0.experiment_id);
    assert_eq!(r1.experiments[1].name, "abc");
    Ok(())
}

#[test]
fn get_experiment_not_found() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r = c.get_experiment("aaaaa");
    assert!(r.unwrap_err().is_resource_does_not_exist());
    Ok(())
}

#[test]
fn get_experiment_by_name() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.get_experiment_by_name("abc")?;
    let e = r1.experiment;
    assert_eq!(e.experiment_id, r0.experiment_id);
    assert_eq!(e.name, "abc");
    Ok(())
}

#[test]
fn get_experiment_by_name_not_found() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r = c.get_experiment_by_name("aaaaa");
    assert!(r.unwrap_err().is_resource_does_not_exist());
    Ok(())
}

#[test]
fn delete_experiment() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;

    let options = SearchExperimentsOptions {
        order_by: &["creation_time"],
        ..Default::default()
    };
    let options_d = SearchExperimentsOptions {
        view_type: ViewType::DeletedOnly,
        ..options
    };

    let max_results = MlflowClient::SEARCH_EXPERIMENTS_MAX_RESULTS_SUPPORTED;
    let page_token = None;
    let a = c.search_experiments(options, max_results, page_token)?;
    assert_eq!(a.experiments.len(), 2);
    let d = c.search_experiments(options_d, max_results, page_token)?;
    assert_eq!(d.experiments.len(), 0);

    c.delete_experiment(&r0.experiment_id)?;

    let a = c.search_experiments(options, max_results, page_token)?;
    assert_eq!(a.experiments.len(), 1);
    let d = c.search_experiments(options_d, max_results, page_token)?;
    assert_eq!(d.experiments.len(), 1);

    Ok(())
}

#[test]
fn restore_experiment() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;

    c.delete_experiment(&r0.experiment_id)?;

    let options = SearchExperimentsOptions {
        order_by: &["creation_time"],
        ..Default::default()
    };
    let options_d = SearchExperimentsOptions {
        view_type: ViewType::DeletedOnly,
        ..options
    };

    let max_results = MlflowClient::SEARCH_EXPERIMENTS_MAX_RESULTS_SUPPORTED;
    let page_token = None;

    let a = c.search_experiments(options, max_results, page_token)?;
    assert_eq!(a.experiments.len(), 1);
    let d = c.search_experiments(options_d, max_results, page_token)?;
    assert_eq!(d.experiments.len(), 1);

    c.restore_experiment(&r0.experiment_id)?;

    let a = c.search_experiments(options, max_results, page_token)?;
    assert_eq!(a.experiments.len(), 2);
    let d = c.search_experiments(options_d, max_results, page_token)?;
    assert_eq!(d.experiments.len(), 0);

    Ok(())
}

#[test]
fn update_experiment() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;

    c.update_experiment(&r0.experiment_id, "def")?;

    let r1 = c.get_experiment(&r0.experiment_id)?;
    let e = r1.experiment;
    assert_eq!(e.experiment_id, r0.experiment_id);
    assert_eq!(e.name, "def");

    Ok(())
}

#[test]
fn create_run() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;

    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    let r2 = c.get_run(&r1.run.info.run_id)?;
    assert_eq!(r2.run.info.run_id, r1.run.info.run_id);
    Ok(())
}

#[test]
fn delete_run() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;

    let options = SearchRunsOptions::default();
    let options_d = SearchRunsOptions {
        run_view_type: ViewType::DeletedOnly,
        ..options
    };

    let max_results = MlflowClient::SEARCH_RUNS_MAX_RESULTS_SUPPORTED;
    let page_token = None;

    let a = c.search_runs(&[&r0.experiment_id], options, max_results, page_token)?;
    assert_eq!(a.runs.len(), 1);
    let d = c.search_runs(&[&r0.experiment_id], options_d, max_results, page_token)?;
    assert_eq!(d.runs.len(), 0);

    c.delete_run(&r1.run.info.run_id)?;

    let a = c.search_runs(&[&r0.experiment_id], options, max_results, page_token)?;
    assert_eq!(a.runs.len(), 0);
    let d = c.search_runs(&[&r0.experiment_id], options_d, max_results, page_token)?;
    assert_eq!(d.runs.len(), 1);
    Ok(())
}

#[test]
fn restore_run() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;

    c.delete_run(&r1.run.info.run_id)?;

    let options = SearchRunsOptions::default();
    let options_d = SearchRunsOptions {
        run_view_type: ViewType::DeletedOnly,
        ..options
    };

    let max_results = MlflowClient::SEARCH_RUNS_MAX_RESULTS_SUPPORTED;
    let page_token = None;

    let a = c.search_runs(&[&r0.experiment_id], options, max_results, page_token)?;
    assert_eq!(a.runs.len(), 0);
    let d = c.search_runs(&[&r0.experiment_id], options_d, max_results, page_token)?;
    assert_eq!(d.runs.len(), 1);

    c.restore_run(&r1.run.info.run_id)?;

    let a = c.search_runs(&[&r0.experiment_id], options, max_results, page_token)?;
    assert_eq!(a.runs.len(), 1);
    let d = c.search_runs(&[&r0.experiment_id], options_d, max_results, page_token)?;
    assert_eq!(d.runs.len(), 0);
    Ok(())
}

#[test]
fn get_run_not_found() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r = c.get_run("aaaaa");
    assert!(r.unwrap_err().is_resource_does_not_exist());
    Ok(())
}

#[test]
fn get_run() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(
        &r0.experiment_id,
        "run1",
        CreateRunOptions {
            ..Default::default()
        },
    )?;
    let r2 = c.get_run(&r1.run.info.run_id)?;
    assert_eq!(r2.run.info.run_id, r1.run.info.run_id);
    assert_eq!(r2.run.info.run_name, "run1");
    Ok(())
}

#[test]
fn log_metric() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    let run_id = &r1.run.info.run_id;
    c.log_metric(run_id, "m1", 1.0, 5.into(), Some(0))?;
    c.log_metric(run_id, "m1", 2.0, 10.into(), Some(1))?;
    let r2 = c.get_run(run_id)?;
    assert!(r2.run.data.metrics.contains(&Metric {
        key: "m1".to_string(),
        value: 2.0,
        timestamp: 10.into(),
        step: Some(1),
    }));
    Ok(())
}

#[test]
fn log_metric_no_step() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    let run_id = &r1.run.info.run_id;
    c.log_metric(run_id, "m1", 1.0, 5.into(), None)?;
    c.log_metric(run_id, "m1", 2.0, 10.into(), None)?;
    let r2 = c.get_run(run_id)?;
    assert!(r2.run.data.metrics.contains(&Metric {
        key: "m1".to_string(),
        value: 2.0,
        timestamp: 10.into(),
        step: Some(0),
    }));
    Ok(())
}

#[test]
fn get_metric_history() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    c.log_metric(&r1.run.info.run_id, "m1", 1.0, 5.into(), Some(0))?;
    c.log_metric(&r1.run.info.run_id, "m1", 2.0, 10.into(), Some(1))?;
    let r2 = c.get_metric_history(&r1.run.info.run_id, "m1", 100, None)?;
    assert_eq!(r2.metrics.len(), 2);
    assert_eq!(r2.metrics[0].key, "m1");
    assert_eq!(r2.metrics[0].value, 1.0);
    assert_eq!(r2.metrics[0].timestamp, 5.into());
    assert_eq!(r2.metrics[0].step, Some(0));
    assert_eq!(r2.metrics[1].key, "m1");
    assert_eq!(r2.metrics[1].value, 2.0);
    assert_eq!(r2.metrics[1].timestamp, 10.into());
    assert_eq!(r2.metrics[1].step, Some(1));
    Ok(())
}

#[test]
fn log_batch() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    let mut metrics = vec![
        Metric {
            key: "m1".to_string(),
            value: 1.0,
            timestamp: 5.into(),
            step: Some(0),
        },
        Metric {
            key: "m2".to_string(),
            value: 2.0,
            timestamp: 10.into(),
            step: Some(1),
        },
    ];
    let mut params = vec![
        Param {
            key: "p1".to_string(),
            value: "v1".to_string(),
        },
        Param {
            key: "p2".to_string(),
            value: "v2".to_string(),
        },
    ];
    let tags = vec![
        RunTag {
            key: "t1".to_string(),
            value: "v1".to_string(),
        },
        RunTag {
            key: "t2".to_string(),
            value: "v2".to_string(),
        },
    ];
    c.log_batch(&r1.run.info.run_id, &metrics, &params, &tags)?;
    let mut r2 = c.get_run(&r1.run.info.run_id)?;
    r2.run.data.metrics.sort();
    metrics.sort();
    assert_eq!(&r2.run.data.metrics, &metrics);
    r2.run.data.params.sort();
    params.sort();
    assert_eq!(&r2.run.data.params, &params);
    assert!(&r2.run.data.tags.contains(&tags[0]));
    assert!(&r2.run.data.tags.contains(&tags[1]));
    Ok(())
}

#[test]
fn log_inputs() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    let dataset_inputs = vec![
        DatasetInput {
            tags: vec![
                InputTag {
                    key: "t1".to_string(),
                    value: "v1".to_string(),
                },
                InputTag {
                    key: "t2".to_string(),
                    value: "v2".to_string(),
                },
            ],
            dataset: Dataset {
                name: "d1".to_string(),
                digest: "aaa".to_string(),
                source_type: "bbb".to_string(),
                source: "ccc".to_string(),
                schema: Some("ddd".to_string()),
                profile: Some("eee".to_string()),
            },
        },
        DatasetInput {
            tags: vec![
                InputTag {
                    key: "t3".to_string(),
                    value: "v3".to_string(),
                },
                InputTag {
                    key: "t4".to_string(),
                    value: "v4".to_string(),
                },
            ],
            dataset: Dataset {
                name: "d2".to_string(),
                digest: "fff".to_string(),
                source_type: "ggg".to_string(),
                source: "hhh".to_string(),
                schema: Some("iii".to_string()),
                profile: Some("jjj".to_string()),
            },
        },
    ];
    c.log_inputs(&r1.run.info.run_id, &dataset_inputs)?;
    let mut r2 = c.get_run(&r1.run.info.run_id)?;
    r2.run.inputs.dataset_inputs.sort();
    r2.run.inputs.dataset_inputs[0].tags.sort();
    r2.run.inputs.dataset_inputs[1].tags.sort();

    assert_eq!(&r2.run.inputs.dataset_inputs, &dataset_inputs);
    Ok(())
}

#[test]
fn set_experiment_tag() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    c.set_experiment_tag(&r0.experiment_id, "t1", "v1")?;
    let r1 = c.get_experiment(&r0.experiment_id)?;
    assert_eq!(r1.experiment.tags.len(), 1);
    assert_eq!(r1.experiment.tags[0].key, "t1");
    assert_eq!(r1.experiment.tags[0].value, "v1");
    Ok(())
}

#[test]
fn set_tag() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    c.set_tag(&r1.run.info.run_id, "t1", "v1")?;
    let r2 = c.get_run(&r1.run.info.run_id)?;
    assert!(r2.run.data.tags.contains(&RunTag {
        key: "t1".to_string(),
        value: "v1".to_string()
    }));
    Ok(())
}

#[test]
fn delete_tag() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    c.set_tag(&r1.run.info.run_id, "t1", "v1")?;
    let r2 = c.get_run(&r1.run.info.run_id)?;
    let tag = RunTag {
        key: "t1".to_string(),
        value: "v1".to_string(),
    };
    assert!(r2.run.data.tags.contains(&tag));
    c.delete_tag(&r1.run.info.run_id, "t1")?;
    let r3 = c.get_run(&r1.run.info.run_id)?;
    assert!(!r3.run.data.tags.contains(&tag));
    Ok(())
}

#[test]
fn log_param() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    c.log_param(&r1.run.info.run_id, "p1", "v1")?;
    let r2 = c.get_run(&r1.run.info.run_id)?;
    assert!(r2.run.data.params.contains(&Param {
        key: "p1".to_string(),
        value: "v1".to_string()
    }));
    Ok(())
}

#[test]
fn search_runs() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;

    let options = SearchRunsOptions::default();
    let max_results = MlflowClient::SEARCH_RUNS_MAX_RESULTS_SUPPORTED;
    let page_token = None;

    let r2 = c.search_runs(&[&r0.experiment_id], options, max_results, page_token)?;
    assert_eq!(r2.runs.len(), 1);
    assert_eq!(r2.runs[0].info.run_id, r1.run.info.run_id);
    Ok(())
}

#[test]
fn update_run() -> Result<()> {
    let s = MlflowServer::start();
    let c = s.mlflow_client();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.create_run(&r0.experiment_id, "", Default::default())?;
    c.update_run(
        &r1.run.info.run_id,
        UpdateRunOptions {
            run_name: Some("def"),
            ..Default::default()
        },
    )?;
    let r2 = c.get_run(&r1.run.info.run_id)?;
    assert_eq!(r2.run.info.run_name, "def");
    Ok(())
}

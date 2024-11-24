use anyhow::Result;
use mlflow_client::data::Timestamp;

use crate::MlflowServer;

#[test]
fn timestamp_now() -> Result<()> {
    let p = MlflowServer::start();
    let c = p.mlflow_client();
    let start = Timestamp::now();
    let r0 = c.create_experiment("abc", Default::default())?;
    let r1 = c.get_experiment(&r0.experiment_id)?;
    let end = Timestamp::now();
    let time = r1.experiment.creation_time;
    assert!(
        start <= time && time <= end,
        "start: {start:?}, time: {time:?}, end: {end:?}"
    );
    Ok(())
}

use std::collections::HashSet;

use anyhow::Result;

use crate::MlflowServer;

#[test]
fn experiments() -> Result<()> {
    let s = MlflowServer::start();
    let m = s.mlflow();
    m.create_experiment("abc", Default::default())?;
    let es = m.experiments()?;
    let a_names = es.iter().map(|e| e.name()).collect::<HashSet<_>>();
    let e_names = HashSet::from_iter(["Default", "abc"]);
    assert_eq!(a_names, e_names);
    Ok(())
}

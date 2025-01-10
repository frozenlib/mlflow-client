use std::{
    fs::{create_dir_all, File},
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
    thread::sleep,
};

use ::mlflow_client::{client::MlflowClient, Mlflow};
use fs2::FileExt;
use tempdir::TempDir;

mod mlflow;
mod mlflow_client;

mod data;

pub struct MlflowServer {
    _dir: TempDir,
    child: Child,
    lock_file: File,
    port: u32,
}
impl MlflowServer {
    pub fn start() -> Self {
        let root = "./target/mlflow";
        create_dir_all(root).unwrap();
        let (port, lock_file) = lock();
        let dir = TempDir::new_in(root, "").unwrap();
        let mut child = Command::new("cargo")
            .args([
                "run",
                "-p",
                "mlflow-runner",
                "--",
                "server",
                "-p",
                &format!("{port}"),
            ])
            .current_dir(dir.path())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let reader = BufReader::new(child.stderr.take().unwrap());
        for line in reader.lines() {
            let line = line.unwrap();
            eprintln!("{line}");
            let msg = if cfg!(windows) {
                format!("INFO:waitress:Serving on http://127.0.0.1:{port}")
            } else {
                format!("[INFO] Listening at: http://127.0.0.1:{port}")
            };
            if line.contains(&msg) {
                break;
            }
        }
        Self {
            child,
            _dir: dir,
            lock_file,
            port,
        }
    }
    pub fn mlflow_client(&self) -> MlflowClient {
        MlflowClient::new(&self.uri()).unwrap()
    }
    pub fn mlflow(&self) -> Mlflow {
        Mlflow::new(&self.uri()).unwrap()
    }
    fn uri(&self) -> String {
        format!("http://127.0.0.1:{}/", self.port)
    }
}
fn lock() -> (u32, File) {
    let start_port = 5100;
    let end_port = 5108;
    loop {
        for port in start_port..end_port {
            let lock_file = File::create(format!("./target/mlflow-{port}.lock")).unwrap();
            if lock_file.try_lock_exclusive().is_ok() {
                return (port, lock_file);
            }
        }
        sleep(std::time::Duration::from_millis(100));
    }
}

impl Drop for MlflowServer {
    #[allow(unstable_name_collisions)]
    fn drop(&mut self) {
        self.child.stdin.take();
        self.child.wait().unwrap();
        sleep(std::time::Duration::from_millis(100));
        self.lock_file.unlock().unwrap();
    }
}

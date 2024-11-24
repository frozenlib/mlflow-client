use std::{
    io::BufRead,
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{sleep, spawn},
    time::Duration,
};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let mut p = Command::new("mlflow")
        .args(&args[1..])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    p.stdin.take();

    let flag = Arc::new(AtomicBool::new(false));
    spawn({
        let flag = flag.clone();
        move || {
            for line in std::io::stdin().lock().lines() {
                if line.is_err() {
                    break;
                }
            }
            flag.store(true, Ordering::SeqCst);
        }
    });
    while p.try_wait().unwrap().is_none() && !flag.load(Ordering::SeqCst) {
        sleep(Duration::from_millis(50));
    }
    if p.try_wait().unwrap().is_none() {
        kill_all(p.id());
        p.wait().unwrap();
    }
    sleep(Duration::from_millis(50));
}

#[cfg(windows)]
fn kill_all(pid: u32) {
    Command::new("taskkill")
        .args(["/F", "/T", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

#[cfg(unix)]
fn kill_all(pid: u32) {
    Command::new("pkill")
        .args(["-P", &pid.to_string()])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

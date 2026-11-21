use bevy::prelude::*;
use serde_json::Value;
use std::{
    env,
    error::Error,
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::{Mutex, mpsc},
    thread,
};

#[derive(Resource, Default)]
pub struct Scenario(pub Value);

#[derive(Resource)]
struct Inbox(Mutex<mpsc::Receiver<Value>>);

pub struct NetPlugin;

impl Plugin for NetPlugin {
    fn build(&self, app: &mut App) {
        let base = env::var("SANTIAGO_ENGINE").unwrap_or_else(|_| "http://127.0.0.1:8790".into());
        let (send, receive) = mpsc::channel();
        thread::spawn(move || {
            if let Err(error) = read_engine(base.trim_end_matches('/'), send) {
                eprintln!("Scenario stream stopped: {error}");
            }
        });
        app.init_resource::<Scenario>()
            .insert_resource(Inbox(Mutex::new(receive)))
            .add_systems(PreUpdate, receive_scenario);
    }
}

fn read_engine(base: &str, send: mpsc::Sender<Value>) -> Result<(), Box<dyn Error>> {
    let latest = Command::new("curl")
        .args(["-fsS", &format!("{base}/api/scenario-latest")])
        .output()?;
    if !latest.status.success() {
        return Err(String::from_utf8_lossy(&latest.stderr).into_owned().into());
    }
    send.send(serde_json::from_slice(&latest.stdout)?)?;

    let mut stream = Command::new("curl")
        .args(["-fsSN", &format!("{base}/api/scenario-events")])
        .stdout(Stdio::piped())
        .spawn()?;
    for line in BufReader::new(stream.stdout.take().unwrap()).lines() {
        if let Some(data) = line?.strip_prefix("data: ") {
            send.send(serde_json::from_str(data)?)?;
        }
    }
    stream.wait()?;
    Ok(())
}

fn receive_scenario(inbox: Res<Inbox>, mut scenario: ResMut<Scenario>) {
    if let Some(newest) = inbox.0.lock().unwrap().try_iter().last()
        && newest["revision"] != scenario.0["revision"]
    {
        scenario.0 = newest;
    }
}

// Write the replay to a video with no visible window: SANTIAGO_RECORD=out.mp4.
// Each frame advances the clock by 1/30 s, so the video does not depend on the render speed.
// The clock stops while building tiles load.
use crate::{city::Buildings, net::Scenario};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::TimeUpdateStrategy,
};
use std::{
    collections::BTreeMap,
    env,
    io::Write,
    process::{Child, Command, Stdio},
    time::Duration,
};

const FPS: f64 = 30.;
const HOLD_SECONDS: f64 = 10.; // Time on screen for the last subtitle.
const WARM_FRAMES: u32 = 15;

#[derive(Resource)]
struct Recorder {
    path: String,
    ffmpeg: Option<Child>,
    // Frames can return from the GPU out of order.
    waiting: BTreeMap<u32, Vec<u8>>,
    written: u32,
}

pub struct RecordPlugin;

impl Plugin for RecordPlugin {
    fn build(&self, app: &mut App) {
        let Ok(path) = env::var("SANTIAGO_RECORD") else { return };
        app.insert_resource(Recorder { path, ffmpeg: None, waiting: BTreeMap::new(), written: 0 })
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO))
            .add_systems(Last, record.run_if(resource_exists::<Buildings>));
    }
}

fn record(
    mut commands: Commands,
    scenario: Res<Scenario>,
    buildings: Res<Buildings>,
    mut step: ResMut<TimeUpdateStrategy>,
    mut advanced: Local<bool>,
    mut frame: Local<u32>,
    mut warm: Local<u32>,
) {
    let Some(last) = scenario.0["scene"]["cues"].as_array().and_then(|cues| cues.last()) else {
        return;
    };
    let total = ((last["after_seconds"].as_f64().unwrap() + HOLD_SECONDS) * FPS) as u32;
    // The city is blank in the first frames, while the GPU prepares the materials.
    if *advanced && *warm < WARM_FRAMES {
        *warm += 1;
    } else if *advanced && *frame < total {
        let index = *frame;
        *frame += 1;
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>,
                  mut recorder: ResMut<Recorder>,
                  mut exit: MessageWriter<AppExit>| {
                let image = event.image.clone().try_into_dynamic().unwrap().into_rgb8();
                let recorder = &mut *recorder;
                let ffmpeg = recorder.ffmpeg.get_or_insert_with(|| {
                    let size = format!("{}x{}", image.width(), image.height());
                    Command::new("ffmpeg")
                        .args(["-y", "-loglevel", "error", "-f", "rawvideo", "-pix_fmt", "rgb24"])
                        .args(["-s", &size, "-r", "30", "-i", "-", "-c:v", "libx264", "-crf", "18"])
                        .args(["-pix_fmt", "yuv420p", "-movflags", "+faststart", &recorder.path])
                        .stdin(Stdio::piped())
                        .spawn()
                        .unwrap()
                });
                recorder.waiting.insert(index, image.into_raw());
                while let Some(bytes) = recorder.waiting.remove(&recorder.written) {
                    ffmpeg.stdin.as_mut().unwrap().write_all(&bytes).unwrap();
                    recorder.written += 1;
                }
                if recorder.written == total {
                    drop(ffmpeg.stdin.take());
                    ffmpeg.wait().unwrap();
                    info!("Wrote {total} frames to {}", recorder.path);
                    exit.write(AppExit::Success);
                }
            },
        );
    }
    *advanced = buildings.ready;
    *step = TimeUpdateStrategy::ManualDuration(if *advanced {
        Duration::from_secs_f64(1. / FPS)
    } else {
        Duration::ZERO
    });
}

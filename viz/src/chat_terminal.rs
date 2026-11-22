use base64::{Engine as _, engine::general_purpose::STANDARD};
use bevy::{
    prelude::*,
    window::{PrimaryWindow, RawHandleWrapper},
};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use serde::Deserialize;
use std::{
    io::{Read, Write},
    sync::mpsc::{Receiver, channel},
    thread,
};
use wry::{
    NewWindowResponse, Rect, WebView, WebViewBuilder,
    dpi::{LogicalPosition, LogicalSize},
};

pub(crate) const ENABLE: &str = "SANTIAGO_EMBEDDED_CHAT";

#[derive(Resource)]
pub(crate) struct ChatReady;

pub(crate) struct ChatTerminal;

impl Plugin for ChatTerminal {
    fn build(&self, app: &mut App) {
        app.insert_non_send(None::<Panel>)
            .add_systems(Update, update.run_if(resource_exists::<ChatReady>));
    }
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Event {
    Ready,
    Toggle,
    Input { text: String },
    Resize { cols: u16, rows: u16 },
}

struct Panel {
    view: WebView,
    child: Box<dyn Child + Send + Sync>,
    pty: Box<dyn MasterPty + Send>,
    input: Box<dyn Write + Send>,
    output: Receiver<Vec<u8>>,
    events: Receiver<Event>,
    window_size: Vec2,
    ready: bool,
    open: bool,
}

impl Panel {
    fn open(
        window_size: Vec2,
        handle: &RawHandleWrapper,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (send_event, events) = channel();
        let html = include_str!("../assets/chat-panel/panel.html")
            .replace(
                "/* XTERM_CSS */",
                include_str!("../assets/chat-panel/xterm.css"),
            )
            .replace(
                "/* XTERM_JS */",
                include_str!("../assets/chat-panel/xterm.js"),
            )
            .replace(
                "/* FIT_JS */",
                include_str!("../assets/chat-panel/addon-fit.js"),
            );
        // This system uses a non-send resource, so the AppKit view is created on the main thread.
        let handle = unsafe { handle.get_handle() };
        let view = WebViewBuilder::new()
            .with_html(html)
            .with_transparent(true)
            .with_navigation_handler(|url| url == "about:blank")
            .with_new_window_req_handler(|_, _| NewWindowResponse::Deny)
            .with_bounds(bounds(window_size, true))
            .with_ipc_handler(move |request| {
                if let Ok(event) = serde_json::from_str(request.body()) {
                    let _ = send_event.send(event);
                }
            })
            .build_as_child(&handle)?;

        let pair = native_pty_system().openpty(pty_size(80, 24))?;
        let mut command = CommandBuilder::new("/bin/bash");
        let root = std::env::current_dir()?;
        command.arg(root.join("tools/city-chat.sh"));
        command.cwd(root);
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        let child = pair.slave.spawn_command(command)?;
        drop(pair.slave);
        let mut reader = pair.master.try_clone_reader()?;
        let input = pair.master.take_writer()?;
        let (send_output, output) = channel();
        thread::spawn(move || {
            let mut buffer = [0; 8192];
            while let Ok(count @ 1..) = reader.read(&mut buffer) {
                if send_output.send(buffer[..count].to_vec()).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            view,
            child,
            pty: pair.master,
            input,
            output,
            events,
            window_size,
            ready: false,
            open: true,
        })
    }

    fn update(&mut self, window_size: Vec2) -> Result<(), Box<dyn std::error::Error>> {
        if self.window_size != window_size {
            self.view.set_bounds(bounds(window_size, self.open))?;
            self.window_size = window_size;
        }
        while let Ok(event) = self.events.try_recv() {
            match event {
                Event::Ready => {
                    self.ready = true;
                    self.update_focus()?;
                }
                Event::Toggle => {
                    self.set_open(!self.open)?;
                }
                Event::Input { text } => self.input.write_all(text.as_bytes())?,
                Event::Resize { cols, rows } => self.pty.resize(pty_size(cols, rows))?,
            }
        }
        if self.ready {
            let output: Vec<u8> = self.output.try_iter().take(32).flatten().collect();
            if !output.is_empty() {
                let data = STANDARD.encode(output);
                self.view
                    .evaluate_script(&format!("window.chatOutput('{data}')"))?;
            }
        }
        Ok(())
    }

    fn set_open(&mut self, open: bool) -> wry::Result<()> {
        self.open = open;
        self.view.set_bounds(bounds(self.window_size, open))?;
        self.update_focus()
    }

    fn update_focus(&self) -> wry::Result<()> {
        if self.open {
            self.view.focus()?;
        }
        self.view
            .evaluate_script(&format!("window.chatOpen({})", self.open))?;
        if !self.open {
            self.view.focus_parent()?;
        }
        Ok(())
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn update(
    window: Single<(&Window, &RawHandleWrapper), With<PrimaryWindow>>,
    mut panel: NonSendMut<Option<Panel>>,
    mut failed: Local<bool>,
    mut replays: MessageReader<crate::history::ReplayStarted>,
) {
    if *failed {
        return;
    }
    let (window, handle) = *window;
    let size = UVec2::new(window.physical_width(), window.physical_height()).as_vec2()
        / window.resolution.base_scale_factor();
    let hide_chat = replays.read().count() != 0;
    let result = match panel.as_mut() {
        Some(panel) => panel.update(size).and_then(|()| {
            if hide_chat && panel.open {
                panel.set_open(false)?;
            }
            Ok(())
        }),
        None => Panel::open(size, handle).map(|new_panel| *panel = Some(new_panel)),
    };
    if let Err(error) = result {
        *failed = true;
        error!("Embedded chat: {error}");
    }
}

fn bounds(size: Vec2, open: bool) -> Rect {
    let inset = 18.;
    let width = if open {
        (size.x * 0.38).max(440.).min(size.x - 2. * inset)
    } else {
        132.
    };
    let height = if open { size.y - 2. * inset } else { 46. };
    Rect {
        position: LogicalPosition::new(size.x - width - inset, inset).into(),
        size: LogicalSize::new(width, height).into(),
    }
}

fn pty_size(cols: u16, rows: u16) -> PtySize {
    PtySize {
        cols,
        rows,
        pixel_width: 0,
        pixel_height: 0,
    }
}

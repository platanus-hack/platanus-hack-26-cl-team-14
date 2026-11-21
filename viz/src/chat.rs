use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    input_focus::{InputFocus, tab_navigation::TabIndex},
    prelude::*,
    text::{EditableText, EditableTextSystems, TextCursorStyle},
    ui::RelativeCursorPosition,
    ui_widgets::TextInput,
    window::Ime,
};
use std::thread::{self, JoinHandle};

pub(crate) struct ChatPlugin;

#[derive(Component)]
pub(crate) struct ChatPanel;

#[derive(Component)]
pub(crate) struct ChatInput;

#[derive(Component)]
struct Transcript;

#[derive(Component)]
struct ChatStatus;

#[derive(Resource)]
struct Chat {
    font: Handle<Font>,
    history: Vec<(String, String)>,
    pending: Option<(String, JoinHandle<Result<String, String>>)>,
}

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_chat)
            .add_systems(Update, (receive_answer, scroll_chat))
            .add_systems(PostUpdate, submit.after(EditableTextSystems));
    }
}

pub(crate) fn city_keys_enabled(
    focus: Res<InputFocus>,
    inputs: Query<(), With<ChatInput>>,
) -> bool {
    !focus.get().is_some_and(|entity| inputs.contains(entity))
}

fn chat_font(font: &Handle<Font>, size: f32) -> TextFont {
    TextFont {
        font: font.clone().into(),
        font_size: FontSize::Px(size),
        ..default()
    }
}

const HINT: &str = "Mensaje · Enter para enviar";

fn setup_chat(mut commands: Commands, mut fonts: ResMut<Assets<Font>>) {
    let font = fonts.add(Font::from_bytes(
        include_bytes!("../assets/barlow/Barlow-Regular.ttf").to_vec(),
    ));
    commands
        .spawn((
            ChatPanel,
            RelativeCursorPosition::default(),
            GlobalZIndex(20),
            Node {
                position_type: PositionType::Absolute,
                right: px(16),
                top: px(16),
                bottom: px(32),
                width: vw(23),
                min_width: px(285),
                max_width: px(440),
                padding: px(18).all(),
                row_gap: px(12),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgb_u8(24, 33, 40)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("Chat"),
                chat_font(&font, 26.),
                TextColor(Color::WHITE),
            ));
            panel
                .spawn((
                    Transcript,
                    ScrollPosition::default(),
                    Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.,
                        min_height: px(0),
                        row_gap: px(12),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                ))
                .with_children(|transcript| {
                    transcript.spawn((
                        Text::new("¿Qué quieres cambiar en Santiago?"),
                        chat_font(&font, 20.),
                        TextColor(Color::srgb_u8(203, 218, 220)),
                    ));
                });
            panel.spawn((
                ChatStatus,
                Text::new(HINT),
                chat_font(&font, 17.),
                TextColor(Color::srgb_u8(180, 222, 210)),
            ));
            panel.spawn((
                ChatInput,
                TextInput,
                TabIndex(0),
                EditableText {
                    allow_newlines: false,
                    ..default()
                },
                TextLayout::no_wrap(),
                chat_font(&font, 21.),
                TextColor(Color::WHITE),
                TextCursorStyle {
                    color: Color::WHITE,
                    ..default()
                },
                Node {
                    height: px(47),
                    width: percent(100),
                    flex_shrink: 0.,
                    padding: UiRect::axes(px(9), px(7)),
                    border: px(1).all(),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BorderColor::from(Color::srgb_u8(142, 171, 170)),
                BackgroundColor(Color::srgb_u8(40, 53, 60)),
            ));
        });
    commands.insert_resource(Chat {
        font,
        history: vec![],
        pending: None,
    });
}

fn add_message(
    commands: &mut Commands,
    transcript: Entity,
    scroll: &mut ScrollPosition,
    font: &Handle<Font>,
    author: &str,
    body: &str,
) {
    commands.entity(transcript).with_children(|list| {
        list.spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(2),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|turn| {
            turn.spawn((
                Text::new(author.to_owned()),
                chat_font(font, 17.),
                TextColor(Color::srgb_u8(164, 215, 203)),
            ));
            turn.spawn((
                Text::new(body.to_owned()),
                chat_font(font, 20.),
                TextLayout {
                    linebreak: LineBreak::WordOrCharacter,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    });
    scroll.0.y = f32::MAX;
}

fn submit(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    focus: Res<InputFocus>,
    mut ime: MessageReader<Ime>,
    scenario: Res<crate::net::Scenario>,
    mut chat: ResMut<Chat>,
    mut inputs: Query<&mut EditableText, With<ChatInput>>,
    mut status: Single<&mut Text, With<ChatStatus>>,
    transcript: Single<(Entity, &mut ScrollPosition), With<Transcript>>,
) {
    let ime_committed = ime
        .read()
        .filter(|e| matches!(e, Ime::Commit { .. }))
        .count()
        != 0;
    if ime_committed
        || !(keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter))
        || chat.pending.is_some()
    {
        return;
    }
    let Some(entity) = focus.get() else { return };
    let Ok(mut field) = inputs.get_mut(entity) else {
        return;
    };
    if field.is_composing() {
        return;
    }
    let prompt = field.value().to_string().trim().to_owned();
    if prompt.is_empty() {
        return;
    }
    field.clear();
    let (list, mut scroll) = transcript.into_inner();
    add_message(&mut commands, list, &mut scroll, &chat.font, "Tú", &prompt);
    status.0 = "Consultando…".into();
    let turns: Vec<_> = chat
        .history
        .iter()
        .map(|(question, answer)| serde_json::json!({"pregunta": question, "respuesta": answer}))
        .collect();
    let context = serde_json::json!({
        "historial": turns,
        "cambios_aplicados": scenario.0.get("applied"),
        "pregunta_actual": &prompt,
    });
    let request = format!(
        "Responde a pregunta_actual usando el historial. Cada llamada a simulate_city parte de la base. Si el usuario amplía el escenario, envía en una sola llamada los cambios_aplicados que desea conservar y los cambios nuevos. Omite el campo de resultado understood. Para un escenario independiente, envía solo los cambios pedidos. Para restablecer la base, envía changes vacío.\n{context}"
    );
    chat.pending = Some((prompt, thread::spawn(move || ask(&request))));
}

fn receive_answer(
    mut commands: Commands,
    mut chat: ResMut<Chat>,
    mut status: Single<&mut Text, With<ChatStatus>>,
    transcript: Single<(Entity, &mut ScrollPosition), With<Transcript>>,
) {
    if !chat
        .pending
        .as_ref()
        .is_some_and(|(_, worker)| worker.is_finished())
    {
        return;
    }
    let (prompt, worker) = chat.pending.take().unwrap();
    let result = worker
        .join()
        .unwrap_or_else(|_| Err("Se interrumpió la consulta.".into()));
    let (list, mut scroll) = transcript.into_inner();
    match result {
        Ok(answer) => {
            add_message(
                &mut commands,
                list,
                &mut scroll,
                &chat.font,
                "Santiago",
                &answer,
            );
            chat.history.push((prompt, answer));
        }
        Err(error) => add_message(&mut commands, list, &mut scroll, &chat.font, "Chat", &error),
    }
    status.0 = HINT.into();
}

fn scroll_chat(
    mut wheels: MessageReader<MouseWheel>,
    panel: Single<&RelativeCursorPosition, With<ChatPanel>>,
    transcript: Single<(&ComputedNode, &mut ScrollPosition), With<Transcript>>,
) {
    let delta: f32 = wheels
        .read()
        .map(|wheel| match wheel.unit {
            MouseScrollUnit::Line => wheel.y * 32.,
            MouseScrollUnit::Pixel => wheel.y,
        })
        .sum();
    if delta == 0. || !panel.cursor_over() {
        return;
    }
    let (node, mut scroll) = transcript.into_inner();
    let scale = node.inverse_scale_factor();
    let limit = (node.content_size().y - node.size().y).max(0.) * scale;
    scroll.0.y = (node.scroll_position.y * scale - delta).clamp(0., limit);
}
fn ask(prompt: &str) -> Result<String, String> {
    use std::{env, io::ErrorKind, path::Path, process::Command};

    const ROLE: &str = "Responde en español como asistente de la maqueta de Santiago. Para preguntas sobre un escenario usa únicamente simulate_city. Pide el dato que falte para interpretar una solicitud ambigua. Usa los valores que devuelve el motor y distingue los cambios no modelados. Responde en pocas frases.";
    let server = Path::new("tools/santiago-mcp/server.mjs")
        .canonicalize()
        .map_err(|_| "Abre la maqueta desde la raíz del repositorio.".to_owned())?;
    let engine = env::var("SANTIAGO_ENGINE").unwrap_or_else(|_| "http://127.0.0.1:8787".to_owned());
    let work = env::temp_dir();
    let selected = env::var("SANTIAGO_CHAT_CLI").ok();
    let candidates: &[&str] = match selected.as_deref() {
        Some("claude") => &["claude"],
        Some("codex") => &["codex"],
        None => &["claude", "codex"],
        _ => return Err("SANTIAGO_CHAT_CLI acepta claude o codex.".into()),
    };

    for &cli in candidates {
        let mut cmd = Command::new(cli);
        cmd.current_dir(&work);
        if cli == "claude" {
            let config = serde_json::json!({"mcpServers":{"santiago":{
                "command":"node", "args":[server], "env":{"SANTIAGO_API_URL":engine}
            }}});
            cmd.args([
                "--print",
                "--output-format",
                "text",
                "--no-session-persistence",
                "--setting-sources",
                "",
                "--settings",
                r#"{"disableAllHooks":true}"#,
                "--disable-slash-commands",
                "--tools",
                "",
                "--strict-mcp-config",
                "--permission-mode",
                "dontAsk",
                "--allowedTools",
                "mcp__santiago__simulate_city",
                "--mcp-config",
                &config.to_string(),
                "--system-prompt",
                ROLE,
            ]);
        } else {
            cmd.args([
                "exec",
                "--ignore-user-config",
                "--ephemeral",
                "--skip-git-repo-check",
                "--sandbox",
                "read-only",
                "--color",
                "never",
                "--disable",
                "shell_tool",
                "--disable",
                "unified_exec",
                "--disable",
                "apps",
                "--disable",
                "plugins",
                "--disable",
                "multi_agent",
                "--disable",
                "multi_agent_v2",
                "--disable",
                "hooks",
                "--disable",
                "memories",
                "-c",
                "approval_policy=\"never\"",
                "-c",
                "web_search=\"disabled\"",
                "-c",
                "project_doc_max_bytes=0",
                "-c",
                "skills.include_instructions=false",
                "-c",
                "mcp_servers.santiago.command=\"node\"",
                "-c",
                "mcp_servers.santiago.required=true",
                "-c",
                "mcp_servers.santiago.enabled_tools=[\"simulate_city\"]",
                "-c",
                "mcp_servers.santiago.tools.simulate_city.approval_mode=\"approve\"",
            ]);
            // JSON string quoting is also valid TOML basic-string quoting for these values.
            cmd.arg("-c").arg(format!(
                "developer_instructions={}",
                serde_json::json!(ROLE)
            ));
            cmd.arg("-c").arg(format!(
                "mcp_servers.santiago.args={}",
                serde_json::json!([server])
            ));
            cmd.arg("-c").arg(format!(
                "mcp_servers.santiago.env.SANTIAGO_API_URL={}",
                serde_json::json!(engine)
            ));
        }
        // End option parsing before the user's text. A prompt is never a shell command.
        let result = match cmd.arg("--").arg(prompt).output() {
            Err(e) if e.kind() == ErrorKind::NotFound => continue,
            Err(e) => return Err(format!("No se pudo abrir {cli}: {e}")),
            Ok(output) => output,
        };
        if !result.status.success() {
            // CLI diagnostics can contain the complete prompt, server config and tool output.
            return Err(format!(
                "{cli} terminó con {}. Revisa su sesión y la conexión con el motor.",
                result.status
            ));
        }
        let answer = String::from_utf8_lossy(&result.stdout).trim().to_owned();
        return if answer.is_empty() {
            Err(format!("{cli} terminó sin respuesta."))
        } else {
            Ok(answer)
        };
    }
    Err("No se encontró el asistente en PATH. Instala o selecciona Claude Code o Codex.".into())
}

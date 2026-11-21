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

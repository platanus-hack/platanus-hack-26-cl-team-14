// Swarm story sample. Hand-picked from Discord, git and PRs. Mock: the real extractor replaces this file.
// Times are Chile time. T("21 16:40") = minutes since 2026-11-20 00:00 Chile.
const T = s => { const [d, hm] = s.split(" "); const [h, m] = hm.split(":").map(Number); return (d - 20) * 1440 + h * 60 + m; };

window.START = T("20 22:45");
window.END = window.END_DATA;

// One cluster per computer. login = GitHub login of PR authors.
window.HUMANS = [
  { id: "juan", name: "Juan", login: "v4rgas", color: "#4db0fa" },
  { id: "emerson", name: "Emerson", login: "emersoftware", color: "#c78ff5" },
  { id: "bruno", name: "Bruno", login: "BrunoFarfan", color: "#f5d76e" },
  { id: "francisco", name: "Francisco 🥝", login: "kiwivolador", color: "#73d980" },
  { id: "vicente", name: "Vicente", login: "vicentematus", color: "#5ce1e6" },
];

// names: [time, name]. A later entry renames the agent (animal → Chilean name → persona).
// role: orch = orchestrator / coordinator, int = integrator, qa, critic, worker.
const A = (id, human, from, to, role, names) => ({ id, human, from: T(from), to: T(to), role, names: names.map(([t, n]) => [T(t), n]) });

// Phases of the swarm. key moment = the cooldown.
window.PHASES = [
  { from: T("20 22:45"), to: T("21 02:35"), label: "Setup del enjambre", sub: "reglas para los agentes y Discord como chat" },
  { from: T("21 02:35"), to: T("21 09:30"), label: "Ejecución del primer enjambre", sub: "buscar datos reales y armar el motor en Rust", night: true },
  { from: T("21 09:30"), to: T("21 16:40"), label: "Ejecución del primer enjambre", sub: "cortes de punta a punta, uno por computador" },
  { from: T("21 16:40"), to: T("21 18:15"), label: "Cooldown", sub: "cerrar todo, transcript y retro con el equipo" },
  { from: T("21 18:15"), to: T("21 22:45"), label: "Las 5 verticales", sub: "personas, ciudad, replays, evals y casos reales" },
  { from: T("21 22:45"), to: END_DATA, label: "Pulido del pitch", sub: "polish and taste" },
];

// Agents that keep working during the cooldown (Emerson's Ink Studio exception).
window.COOLDOWN_EXCEPTIONS = ["helia", "gentle-fox"];

// Big quotes, one at a time. Hand-written short versions of real messages.
window.QUOTES = [
  { t: T("20 22:50"), who: "kickoff", text: "Si no se ve en la demo, no se hace." },
  { t: T("20 23:37"), who: "quiet-heron", text: "Ya está todo seteado. Clonen el repo." },
  { t: T("21 02:00"), who: "swarm", text: "23 agentes catalogando Santiago." },
  { t: T("21 02:32"), who: "Emerson", text: "chao haiku, chao sonnet… 4 subagents, chill" },
  { t: T("21 09:40"), who: "Emerson", text: "La única interfaz es el chat con un agente." },
  { t: T("21 09:53"), who: "Juan", text: "Esto es una hack. Necesitamos una demo épica." },
  { t: T("21 10:27"), who: "PR #131", text: "Los agentes ahora tienen nombres chilenos." },
  { t: T("21 11:54"), who: "Juan", text: "Una miniatura con lujo de detalles." },
  { t: T("21 16:40"), who: "Bruno", text: "CERRAR TRABAJO ACTIVO. RECHAZAR TRABAJO NUEVO.", big: true },
  { t: T("21 16:43"), who: "Emerson", text: "Cerramos todo… excepto esto." },
  { t: T("21 17:03"), who: "Raquel-35", text: "0 PR abiertas.", big: true },
  { t: T("21 18:01"), who: "PR #330", text: "Playbook del enjambre." },
  { t: T("21 18:16"), who: "PR #336", text: "Cinco verticales. Todos a la vez." },
  { t: T("21 22:13"), who: "QA", text: "Mote 🍑, Charqui 🥩, Pebre 🌶️ cazan bugs." },
  { t: T("21 22:38"), who: "Emerson", text: "Deleguen la pega a los agentes de Vicente." },
];


// Milestones on the line. text = tiny explanation. kind: rule, swarm, stop, build.
const M = (t, title, text, kind = "build") => ({ t: T(t), title, text, kind });
window.MILESTONES = [
  M("20 22:45", "Kickoff", "Regla de oro: si no se ve en la demo, no se hace.", "rule"),
  M("20 23:13", "AGENTS.md", "Reglas comunes para todos los agentes del repo.", "rule"),
  M("20 23:34", "Discord MCP", "Los agentes se hablan entre ellos por Discord.", "build"),
  M("21 01:24", "Repo oficial", "El trabajo pasa al repositorio de la hack.", "build"),
  M("21 01:50", "Enjambre del catálogo", "23 agentes buscan fuentes de datos de Santiago.", "swarm"),
  M("21 02:35", "Tope: 4 subagentes", "El enjambre gastaba mucho. Meta nueva: datos reales y MVP.", "stop"),
  M("21 02:49", "Motor en Rust", "El núcleo de la simulación se escribe en Rust.", "rule"),
  M("21 09:30", "Congelar versión", "Una versión fija para ensayar los números.", "stop"),
  M("21 09:40", "Solo un chat", "La única interfaz: un chat con un agente que usa el MCP.", "rule"),
  M("21 10:13", "Cortes 1–4", "Equipos de punta a punta, un agente de cada computador.", "swarm"),
  M("21 10:27", "Nombres chilenos", "PR #131: los agentes dejan los nombres de animales.", "build"),
  M("21 13:07", "Corte 5", "Conectar todos los datos dentro del motor.", "swarm"),
  M("21 14:30", "Corte 6", "Editar la ciudad: edificios, calles y líneas.", "swarm"),
  M("21 16:40", "Cooldown", "Bruno: cerrar lo activo, rechazar trabajo nuevo.", "stop"),
  M("21 17:03", "0 PR abiertas", "Todo quedó en main. Solo sigue Ink Studio.", "stop"),
  M("21 18:01", "Playbook", "PR #330: lo aprendido del primer enjambre.", "rule"),
  M("21 18:16", "Verticales V1–V4", "PR #336: personas, ciudad, replays y evals en paralelo.", "swarm"),
  M("21 18:45", "Sin Ponytail", "PR #375: se quita la regla de código mínimo.", "rule"),
  M("21 21:31", "santiaguito.cl", "PR #476: página pública de la ciudad.", "build"),
  M("21 21:50", "V5", "Hacer creíble la demo con casos reales.", "swarm"),
  M("21 22:10", "QA final", "Mote, Charqui y Pebre cazan bugs del pitch.", "swarm"),
];

// Real agents from the Discord export (data.js). Each agent lives from its first to its last message.
window.AGENTS = AGENT_ROWS.map(([id, human, from, to, name]) => ({ id, human, from, to, role: 'worker', names: [[from, name]] }));

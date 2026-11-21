// One shared bot reaction acknowledges receipt, not an agent's answer.
export async function startReceipts(token, discord, addReaction) {
  const guild = "1573514576400752640";
  const channels = new Set(["1573561159955648553", "1573514577109581907"]);
  const enabledAt = Date.now();
  const { url } = await discord("/gateway");

  function connect() {
    const socket = new WebSocket(`${url}/?v=10&encoding=json`);
    let sequence = null;
    let acknowledged = true;
    let timer;
    let firstBeat;
    const send = (op, d) => socket.send(JSON.stringify({ op, d }));
    const heartbeat = () => {
      if (!acknowledged) return socket.close(4000);
      acknowledged = false;
      send(1, sequence);
    };

    socket.addEventListener("message", ({ data }) => {
      const event = JSON.parse(data);
      if (event.s !== null && event.s !== undefined) sequence = event.s;
      if (event.op === 10) {
        firstBeat = setTimeout(() => {
          heartbeat();
          timer = setInterval(heartbeat, event.d.heartbeat_interval);
        }, Math.random() * event.d.heartbeat_interval);
        send(2, { token, intents: 512, properties: {
          os: process.platform, browser: "team-receipts", device: "team-receipts",
        } });
      }
      if (event.op === 11) acknowledged = true;
      if (event.op === 1) send(1, sequence);
      if (event.op === 7 || event.op === 9) socket.close(4000);
      if (event.t === "READY") console.error("Human receipts ready: general and agent-chat only.");
      if (event.t !== "MESSAGE_CREATE") return;
      const message = event.d;
      if (message.guild_id !== guild || !channels.has(message.channel_id)) return;
      if (message.author.bot || message.webhook_id || ![0, 19].includes(message.type)) return;
      const createdAt = Date.parse(message.timestamp);
      if (createdAt < enabledAt) return;
      void addReaction({ channel_id: message.channel_id, message_id: message.id, emoji: "👀" })
        .then(() => console.error(JSON.stringify({
          receipt: message.id, channel: message.channel_id,
          latency_ms: Date.now() - createdAt,
        })))
        .catch(() => console.error(`Receipt failed for message ${message.id}.`));
    });
    socket.addEventListener("error", () => console.error("Receipt Gateway connection error."));
    socket.addEventListener("close", ({ code }) => {
      clearTimeout(firstBeat);
      clearInterval(timer);
      console.error(`Receipt Gateway closed: ${code}.`);
      if ([4004, 4010, 4011, 4012, 4013, 4014].includes(code)) process.exit(1);
      setTimeout(connect, 5000);
    });
  }
  connect();
}

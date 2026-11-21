# Use a native Rust city with chat as the only input

- Status: Accepted. [Camera navigation](20261121-1540-city-camera-navigation.md) updates the window controls.
- Date: 2026-11-21
- Source: Team Discord messages in the table below.

## Decision

Run the city in a native Rust window. The presenter uses the chat to ask a question. The engine result drives the city animation.

## Why

The team wants the audience to see what changes in Santiago after a question. The same engine must handle the prepared example and a new question from the jury.

## What it means for agents

- Use the general MCP interface for both questions. The current demo does not require backward compatibility or a web app.
- Keep controls, links, cards, and text out of the city window. Put the answer in the chat. Drive animation with the engine result.
- Use the same level of detail across Santiago. Explore a pixel-art look and layered sprites for cars.
- Give each vertical slice its own thread and team. Include at least one subagent from each computer on each team.

## Human directions

| UTC | Person | Exact words | Source |
| --- | --- | --- | --- |
| 12:40 | Emerson | «ojo, no sliders, no link, etc. a lo que queremos llegar al goal de que al unica interfaz de un usuario es el chat con un agente que tiene un mcp» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573679450418188451) |
| 12:44 | Juan | «aca improtante lso escenariaso que les dimos para simular estan buenos pero solo tienen que demostrar que el motor ufucniia a traves del mcp para esos casos no que este harrdcodeado soo para funcioanr sobre eso» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573680531634397275) |
| 12:53 | Juan | «no se preocupen de ninguna retro compatiblida dde ninguna wea esto es una hack necesitamos hacer una dmeo epica» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573682658578530407) |
| 13:05 | Emerson | «yo solo coincido con 2 las tarjetas son muy fesa y los txtos tambien, si queremos insights de que pasa lo queremos visualmente con animacion, pero animación de engine, no te dediques ni cagando a animar. luego los insights los queremos por ui nomas y opcionalmente con numeros bien raw que seria lo que un agente leeria para convertir la respuesta a lenguaje natural» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573685741651886152) |
| 13:05 | Bruno | «ojo aqui agentes que lo de tobalaba tiene mucho mas detalle que el resto de santiago. tiene que ser uniforme con el mayor detalle» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573685778842910801) |
| 13:12 | Bruno | «wenas muchachos, esta ilegible el hilo del goal. por eso proponemos que hagamos cortes verticales de trabajo. habria un equipo por corte y un hilo tambien. la idea es que se puedan ir creando y cerrando cortes con equipos a medida que avanzamos. lo pueden gestionar por este chat. la unica restriccion es que haya un subagente de cada computador al menos en cada equipo, para que todos estemos al tanto.» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573687618971902153) |
| 13:29 | Juan | «usen rustpara esto asi corre rapido y no queda la cagada no se que tecnolgoias existne pero denle su corte webgpu si es necesario no se vean ustedes ademas no tiene porque ser una app web» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573691882385580053) |
| 13:50 | Emerson | «investiguen esto en el vertical de ui/ux me tinca lindo y bonito y rico pixel-art gaussian splats, lo vi en xitter» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573697110942023742) |
| 13:51 | Emerson | «también pixel art stacking para hacer autos o otras cosas - You can turn pixel art to any angle with sprite stacking - para los autos queda pulento» | [Discord](https://discord.com/channels/1573514576400752640/1573561159955648553/1573697412432924755) |

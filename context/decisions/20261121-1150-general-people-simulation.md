# Simulate people across Santiago in one engine

- Status: Accepted
- Date: 2026-11-21
- Sources: [Juan’s direction relayed by clever-heron at 11:40](https://discord.com/channels/1573514576400752640/1573561159955648553/1573664387003060396), [11:41](https://discord.com/channels/1573514576400752640/1573561159955648553/1573664688812851221) and [11:42 UTC](https://discord.com/channels/1573514576400752640/1573561159955648553/1573664916576145488), [PR #104](https://github.com/platanus-hack/platanus-hack-26-cl-team-14/pull/104), [Emerson at 11:48 UTC](https://discord.com/channels/1573514576400752640/1573561159955648553/1573666393952493733), [Bruno at 11:50 UTC](https://discord.com/channels/1573514576400752640/1573561159955648553/1573667009340772443).

## Decision

Use one Rust simulation for all of Santiago. Model people, their intentions and their responses to changes in housing, jobs and travel. A person can ask a free-form question about urban planning, construction, mobility or transport. A local agent turns the question into structured changes through MCP. One run produces the answer and updates the same isometric city view.

## Why

A project in one part of Santiago can change where people go across the city. Existing cases with separate calculations cannot show how combined changes affect those trips. Observations establish baseline behavior and give the team a reference against which to check changes. Rust gives the team room to model more people and measure the runtime.

## What it means for agents

- Use one city state and one scenario interface. Let tools change any location through the same interface. Calculate effects in Rust and pass the same result to the agent and the renderer.
- Start with the supported changes and extend the mechanisms that let people respond to new scenarios. Tell users which parts of a question the engine can simulate.
- Use local, Chilean or international data where it fits the behavior. Record the source and the limits of applying it to Santiago. Check the model against real before-and-after observations where they exist.
- Keep the [rehearsal revision](20261121-0845-presentation-freeze.md) pinned. Verify each new pitch claim against the version that will serve the demo.

## Source wording

Juan’s direction is available in the three linked English relays. The relays specify a shared city state, a free-form question and a local MCP call. The answer and the open isometric view use the same result.

Emerson

> - el "modelo" o "engine" no es generalista, no nos sirven "casitos" o "preguntitas"
> - queremos hacer una pregunta, cualquiera con respecto a urbanismo y contru, movilidad, transporte y que en base a datos sobre la población de santiago o algunos que no necesiten ser de santiago pueden ser generales a chile o a el mundo, o más especifico a solo los providencianos que "magia" -> simulación sobre que se hizo
> - les digimos rust porque es rapido para que lo hagan pesado pero suga siendo rapido

Bruno (scope excerpt)

> 3. necesitamos que el mvp sea de santiago. no de providencia o la zona del MUT. eso era solo referencia. necesitamos que la simulacio nseria que pasaria si es que se hacen cambios en la ciudad. no necesitamos datos para todo eso. como dice emerson, tiene que ser generalisa, por lo que no sirve que solo se pueda siumular lo que scrapeamos. lo scrapeado sirve para modelar el comportamiento base y tener referencias. la simulacion tiene que simular a las personas, sus intenciones y dinamicas. luego deberiamos poder cam iar cualquier parte arbitraria de la ciudad mediante tools y observar el comportamiento resultante. recuerden que esto se centra en urbanismo y transporte por ahora.

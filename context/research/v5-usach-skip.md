# Universidad de Santiago Line 1 skip, 21 April 2025

This comparison checks how the Metro entry model responds when trains pass one station and serve its neighbors. It supports the Metro scene. It uses an event separate from the 18 November 2024 Line 1 development case.

Metro announced a station closure at 19:20 on Monday 21 April. The report says trains continued to skip Universidad de Santiago through the end of that day's service. Bins starting at 19:30 and 20:00 are inside the closure. [Operator notice reported by En Cancha](https://www.encancha.cl/enlahora/movilidad/2025/04/21/metro-anuncia-el-cierre-de-estacion-universidad-de-santiago/)

I froze the two bins and the two neighboring stations before I opened the counts. Thursday 24 April is the same-week reference. Bins at 18:00 and 18:30 provide a same-station check on differences before the closure. The weekday mismatch limits the comparison. The model uses a normal weekday and the packed network. It does not simulate the incident or any associated travel disruption. [Frozen comparison](https://discord.com/channels/1573514576400752640/1573863187672207411/1573879972739682426)

## Observed and modeled entries

Each row is one half-hour of Metro boarding stages. Every selected DTPM row has expansion factor 1. The engine change is the difference between the closure and baseline outputs.

| Station | Local time | Thursday | Monday | Observed change | Modeled change |
| --- | ---: | ---: | ---: | ---: | ---: |
| Estación Central | 19:30 | 1,257 | 1,619 | +362 | +237.8 |
| Estación Central | 20:00 | 1,107 | 1,407 | +300 | +230.8 |
| San Alberto Hurtado | 19:30 | 362 | 429 | +67 | +94.1 |
| San Alberto Hurtado | 20:00 | 309 | 406 | +97 | +88.4 |
| Universidad de Santiago (closed) | 19:30 | 655 | 0 | −655 | −418.3 |
| Universidad de Santiago (closed) | 20:00 | 599 | 0 | −599 | −406.0 |

The four open-station cells give **0.0755** for the model (229.1 / 3,035) and **0.2722** for no change (826 / 3,035). The numerator sums absolute differences between predicted and observed changes. The denominator sums the Thursday entries. Observed and modeled entries at the closed station are zero. Those cells stay outside the score.

In the pre-closure bins, Monday minus Thursday was +326 and +54 at Estación Central and −7 and −23 at San Alberto Hurtado. The same no-change error formula gives 410 / 7,037 = 0.0583 in those bins. Monday was already higher at Estación Central. This is a descriptive comparison on a separate event. The station profiles use April 2026 boardings. This run used the existing access rule and no 21 April 2025 counts as inputs. The day-to-day differences limit any estimate of the causal effect. [Engine data](../../engine/README.md)

## Final-engine rerun

Engine: `b611722bc503472996480f0fb55bc95829b7195d`. The four cells
reproduced the table above without a new fit. The model error is
229.1 / 3,035 (7.55%). No change gives 826 / 3,035 (27.22%).
The two closed-station bins are zero and stay outside the score.

Use the command in [Reproduce](#reproduce) with a release `scenario`
binary from this SHA. See the [run receipt](https://discord.com/channels/1573514576400752640/1573943617813348484/1573944733146021968).
The run used binary SHA-256 `cc51651abdabbd054f1c690b221d6e33dfb5e86530e1fe715e90a6ae385dd9e4`.

## Reproduce

Use the two dated stage members from the DTPM ZIP. Count rows with `tipo_transporte = METRO`, the local date in `tiempo_subida`, the bin in `media_hora_subida`, and the listed station in `parada_subida`. The small boarding workbook pools 21–25 April as a weekday average. [DTPM catalog](https://www.dtpm.cl/index.php/documentos/matrices-de-viaje) · [Stages ZIP](https://www.dtpm.cl/descargas/modelos_y_matrices/Tabla-de-etapas-011025.zip)

Run the release `scenario` binary at `b611722bc503472996480f0fb55bc95829b7195d` with its packed data.
The original run used `84545dfca20d77127b53cd186a6d205bb7ee9f11`.

```sh
printf '%s\n' '[{"kind":"station","op":"set","target":{"type":"station","id":"US"},"value":0,"half_hours":[39,41],"temporary":true}]' |
  engine/target/release/scenario data/mvp - > usach.json
jq '.stations[] | select(.id == "EL" or .id == "AH" or .id == "US") |
    {id, base: .base_half_hour[39:41], event: .scenario_half_hour[39:41]}' usach.json
```

Run the [attached scorer](https://discord.com/channels/1573514576400752640/1573943617813348484/1573944809926828082)
with the full `usach.json` output.

The request closes the entrance for both bins and keeps service at all other stations. This approximates a skip for entry counts. It keeps the travel time through the station fixed.

### Source checks

| ZIP member | SHA-256 of the gzip member |
| --- | --- |
| `etapas/2025-04-21.etapas.csv.gz` | `09d1ba7d6022373714e8f933d9d00366ce435dc8fc15206bf4f2290b0ac961b1` |
| `etapas/2025-04-24.etapas.csv.gz` | `093da57cbd0db74bcb817a3659aaba3e03bbff990062c40f503c764824eb23bd` |

The byte ranges of the compressed ZIP members are 162–334876014 and 1012991155–1354193118. Both members decompressed to completion. With the full archive, use `unzip -p` to get the gzip member. To count the cells after `gzip -dc`:

```sh
awk -F '|' -v date="$dtpm_date" '
  $6 == "METRO" && substr($10, 1, 10) == date &&
  $23 ~ /^(ESTACION CENTRAL|SAN ALBERTO HURTADO|UNIVERSIDAD DE SANTIAGO)$/ {
    time = substr($13, 1, 5)
    if (time ~ /^(18:00|18:30|19:30|20:00)$/) count[$23, time]++
  }
  END { for (cell in count) { split(cell, key, SUBSEP); print key[1], key[2], count[cell] } }
'
```

An absent station and time pair in this count has zero recorded entries.

# Attendance at entertainment and sporting events

Chile's II ENUT 2023 includes attendance as a member of the audience at cultural, entertainment, or sporting events. Question VS2 combines these event types. It gives one participation flag and one time measure for each reference day. It does not identify football, the venue, or the event. [INE questionnaire, question VS2](https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/formularios/ii-enut/cuestionario-de-uso-del-tiempo-ii-enut-2023.pdf?sfvrsn=8734c5f5_4)

## Santiago reference

In the survey's urban Metropolitan Region, 4,191 respondents aged 12 or older answered the time-use questionnaire. I used the assigned weekday and assigned weekend day.

| Reference day | Yes | No | Missing | Weighted participation |
| --- | ---: | ---: | ---: | ---: |
| Weekday | 110 | 4,081 | 0 | 2.69% |
| Weekend day | 230 | 3,958 | 3 | 5.81% |

To reproduce, read the public CSV v2. Select `region_ord = 7`, `tiempo = 1`, and `edad >= 12`. Use `vs2_p_ds` for the weekday and `vs2_p_fds` for the weekend day. Divide the sum of `fe_cut` for code 1 by the sum for codes 1 and 2. Code 96 is missing. The archive SHA-256 is `712a270e3e3f0d7fd6675418c66631e31803f0d9d33ca949beb13988d3b0c8b0`. The same calculation nationwide rounds to the published 2.9% and 6.1%. The regional figures are weighted sample estimates. [INE CSV v2](https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/bbdd/ii-enut/250403-ii-enut-bdd-csv-v2.zip?sfvrsn=ed041caf_7) · [Data manual](https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/manuales/ii-enut/manual-de-uso-de-base-de-datos-ii-enut-2023.pdf) · [Results, tables 38–39](https://www.ine.gob.cl/docs/default-source/uso-del-tiempo-tiempo-libre/publicaciones-y-anuarios/ii-enut/informe-de-principales-resultados-ii-enut-2023.pdf)

Use this reference to check broad leisure participation on matching day types. Keep confirmed match attendance as a separate input for the stadium replay. A model of whether someone goes to the match or chooses other leisure still needs assumptions about interest, ticket access, and travel. [Monumental match attendance evidence](monumental-match-attendance.md)

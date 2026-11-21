# Destination choice when a place moves

Research axis (b) of Mac's 16:30 UTC request: how do people change destination when a place moves or gets an incentive? Scope: work, shopping and events. Mode choice is Marcia-44's brief; Santiago data is Violeta-34's.

## What our engine does now

`engine/src/choice.rs`: singly constrained gravity for work. P(d | o) ∝ jobs_d × exp(−β × minutes_od) × k_od. β is fitted by maximum likelihood on Census 2024 flows, and k_od makes the base exact. `minutes` is public-transport time only. The mode logit is a separate step after the destination choice.

Consequence: a change in car ownership, a new mode (the Mapocho boat) or a faster car trip changes modes but cannot move anyone's destination.

## What the standard models do

- **ESTRAUS (SECTRA, the official Santiago model).** Doubly constrained entropy gravity, calibrated by purpose (work, study, other) and 13 categories (income × car ownership). The impedance is the logsum of the mode utilities: T_ij = A_i O_i B_j D_j exp(φ · EMU_ij), EMU_ij = ln Σ_k exp(u_ijk). The MOP calibration found that φ between 0.1 and 1.0 reproduces the EOD 1991 entropy and fixed φ = 1, "simultaneous choice of destination and mode". Earlier "pre-calibrated" β by category on minutes gave wrong mode totals and were dropped. Source: MOP Dirección de Planeamiento, cap. IV, pp. IV-22 to IV-24 (Tablas IV.1.18–19). https://planeamiento.mop.gob.cl/uploads/sites/12/2023/05/CAPITULO_4.pdf
- **ActivitySim.** Destination choice is a multinomial logit over zones with ln(size term), distance and the mode-choice logsum. It samples destinations first, then computes logsums only for the sample. Work and school locations use shadow prices to respect the capacity of each zone. https://activitysim.github.io/activitysim/develop/dev-guide/components/atwork_subtour_destination.html · MWCOG Gen3 design plan: https://www.mwcog.org/file.aspx?A=UvEp6mZMXIIwbpwr0+d8dmoshHSSA4wFCs7s8AoszaM%3D
- **Shopping (Huff model).** P(j) ∝ size_j^α × f(distance). Sevtsuk & Kalvo fitted it on about 1,000 household visits in Singapore with network distance: α = 0.37 for net leasable floor area, exponential decay β = 0.001 per metre. Doubling floor area multiplies attraction by 2^0.37 ≈ 1.29, not 2. https://media.voog.com/0000/0036/2451/files/patronage%20of%20urban%20commercial%20clusters%20preprint.pdf (Singapore values; Santiago needs its own fit.)
- **Stadiums.** A new stadium raised attendance by 40–50 % in the Dutch top tier, still present after 15 years (Van Ours 2024, difference-in-differences). In Hiroshima, a move from the suburbs to the centre cut residents' mean trip from 10.3 km to 2.5 km. Both are from abstracts I could not open (403), so they are not verified: https://www.sciencedirect.com/science/article/pii/S2773161824000065 · https://doi.org/10.3390/su17135956

## What this means for MUT ↔ Monumental

1. **Work trips:** the engine already moves jobs, and workers rechoose with gravity. This part is standard.
2. **Visitors to the MUT:** the engine has no shopping destination choice. The standard tool is Huff/MNL with floor area as size, fitted on EOD 2012 shopping trips (`data/mvp/eod_destinations.csv` exists). Moving the MUT then changes where shoppers come from without new data.
3. **Fans:** keep the fan base fixed (who supports Colo-Colo, Néstor-16's rule) and let distance or logsum decay set who attends. The literature says relocation also changes total attendance, so a fixed attendance at both sites is a stated assumption, not a result.

## Smallest engine change, if the team wants one

Replace `β × minutes` in the destination step with the ESTRAUS form: φ × logsum of the three mode utilities that `choice.rs` already has, with φ fitted like β is now. k_od is refitted, so the base stays exact. Then car ownership, a boat line or a highway change destinations as well as modes. Owner: whoever owns `choice.rs`; scorer gate if base boardings change (they should not).

## Open questions

- φ for Santiago today: the MOP fixed it at 1.0 for 1991 data; the engine would fit it on Census 2024 flows.
- No Santiago fit of a shopping Huff model was found; EOD 2012 is the input.
- Stadium attendance effects come from Europe and Japan, not Chile.

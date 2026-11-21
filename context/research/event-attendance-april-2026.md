# April 11 event attendance comparison

At 13,000 attendees, the fixed model gives **1,719.76 boardings** at Parque O’Higgins. The measured positive contrast was **3,399.87**, or 1,680.11 more. Red Bull reported more than 13,000 spectators at the Movistar Arena final. La Tercera reported about 13,000. The audited attendance and end time remain unknown. The ticket listed a 17:00 start. [Red Bull](https://www.redbull.com/es-es/el-menor-campeon-final-internacional-batalla-2026) · [La Tercera](https://www.latercera.com/culto/noticia/campeon-en-casa-el-menor-se-quedo-con-la-final-internacional-de-red-bull-batalla/) · [Ticket listing](https://www.puntoticket.com/red-bull-batalla-internacional-2026)

The team registered the historical calculation before running the model. Among 360,648 workers employed in Santiago comuna, 63.976% had Census public-transport mode. The 2012 Santiago EOD normal-Saturday survey gave 1,154,358.09 expanded trips with any Metro stage out of 3,638,246.58 with only public stages (869 of 2,862 survey trips). At the arena building centroid, 65.172% of the modeled walk-access weight went to Parque O’Higgins. Multiplying the three shares gives **0.132288935 boardings per attendee**. [Registration](https://discord.com/channels/1573514576400752640/1573579088684253235/1573682272270417961) · [EOD aggregate](../../data/mvp/saturday_public_metro.json) · [Arena footprint, OSM way 32864171](https://www.openstreetmap.org/way/32864171)

The numerator sums the positive station contrast against its 25 Line 2 peers. Nine half-hours had data in the fixed 19:00–23:30 window. The 23:30 bin had no observation. Dividing 3,399.8650629 by 0.132288935 gives a **conditional equivalent of 25,700 attendees**. This calculation assumes that the event caused all the positive contrast and that the nine observed bins captured all event-related boardings at this station. Neither assumption has independent confirmation. The reported crowd size provides a separate comparison. [Contrast definition and source coverage](extra-event-contract.md)

## Run the general model

The general CLI returns mode totals, return-comuna totals and networkwide Metro access for the whole event. The end half-hour is supplied metadata. The station comparison above applies only to the completed April event.

```sh
echo '{"longitude":-70.718561,"latitude":-33.495861,"venue_cut":"13102","attendance":1000,"assumed_end_half_hour":40}' > /tmp/attendance-request.json
cargo run --release --manifest-path engine/Cargo.toml --bin event_attendance -- data/mvp /tmp/attendance-request.json
```

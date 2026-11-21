# Santiago landmarks at city scale

The V2 mock gives the audience a fixed orientation before the replay zooms in. It uses the repository's terrain, census block centers, and mapped positions for the Mapocho, the tower, and Cerro San Cristóbal. The Plaza Italia point uses the Plaza Baquedano coordinates in Wikidata. [Terrain source](../../web/data/terrain.json) · [Landmark data](../../web/data/santiago_landmarks.geojson) · [Plaza Baquedano](https://www.wikidata.org/wiki/Q2415377)

Esri recommends placing labels close to their features and shifting labels to avoid overlaps. This mock uses three short callouts and labels the river beside its path. [Precise label placement](https://www.esri.com/about/newsroom/arcwatch/precise-label-placement)

Preview from the repository root with `python3 -m http.server 43152`. Open <http://localhost:43152/pitch/santiago-landmarks.html>.

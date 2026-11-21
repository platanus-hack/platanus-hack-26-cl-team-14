LOAD spatial;
SET threads=3;
SET memory_limit='2GB';
CREATE TEMP TABLE actual AS
SELECT tile, "column" AS col, "row" AS row_id, layer, kind, structure, "use" AS road_use,
       geom::GEOMETRY AS geom FROM ST_Read('data/raw/road-surface/published.geojson');
SELECT count(*) AS decoded_polygons,count(DISTINCT tile) AS tiles,sum(ST_NumInteriorRings(a.geom)) AS holes,
       countif(NOT ST_IsValid(a.geom)) AS invalid_polygons,
       countif(ST_GeometryType(a.geom)<>'POLYGON') AS other_geometry,
       countif(NOT ST_Covers(t.geom,a.geom)) AS outside_tile,
       countif(t.tile IS NULL OR tile<>a.row_id*20+a.col) AS bad_tile_ids
FROM actual a LEFT JOIN tiles t USING(tile);
-- Compare the two sides of every interior tile boundary at the serialized precision.
WITH seams AS (
    SELECT a.tile AS left_id,b.tile AS right_id,
           ST_Intersection(ST_Boundary(a.geom),ST_Boundary(b.geom)) AS line
    FROM tiles a JOIN tiles b ON (a.row_id=b.row_id AND b.col=a.col+1)
                              OR (a.col=b.col AND b.row_id=a.row_id+1)
), sides AS (
    SELECT s.left_id,s.right_id,a.structure,a.road_use,a.layer,a.tile,
           ST_Union_Agg(ST_CollectionExtract(ST_Intersection(a.geom,s.line),2)) AS edge
    FROM seams s JOIN actual a ON a.tile IN (s.left_id,s.right_id) AND ST_Intersects(s.line,a.geom)
    GROUP BY s.left_id,s.right_id,a.structure,a.road_use,a.layer,a.tile
), shared AS (
    SELECT left_id,right_id,structure,road_use,layer,
           coalesce(first(edge) FILTER (WHERE tile=left_id),ST_GeomFromText('LINESTRING EMPTY')) AS lhs,
           coalesce(first(edge) FILTER (WHERE tile=right_id),ST_GeomFromText('LINESTRING EMPTY')) AS rhs
    FROM sides GROUP BY left_id,right_id,structure,road_use,layer
)
SELECT round(coalesce(sum(ST_Length(meters(ST_Difference(lhs,rhs))) + ST_Length(meters(ST_Difference(rhs,lhs)))),0),3) AS unmatched_seam_metres
FROM shared;
CREATE TEMP TABLE cameras AS
SELECT camera, ST_Buffer(meters(ST_Point(lon,lat)),radius_m,32) AS g
FROM read_csv('data/raw/road-surface/cameras.csv',header=true,
              columns={'camera':'VARCHAR','lon':'DOUBLE','lat':'DOUBLE','radius_m':'DOUBLE'});
CREATE TEMP TABLE original AS SELECT width_m,meters(ST_GeomFromGeoJSON(geometry)) AS g
FROM read_csv('data/raw/road-surface/baseline.csv',header=true,columns={'width_m':'DOUBLE','geometry':'VARCHAR'});
CREATE TEMP TABLE route AS SELECT mode,source,route_id,meters(ST_GeomFromGeoJSON(geometry)) AS g
FROM read_csv('data/raw/road-surface/routes.csv',header=true,
              columns={'mode':'VARCHAR','source':'VARCHAR','route_id':'VARCHAR','geometry':'VARCHAR'});
SELECT c.camera,w.highway,w.width_source,count(*) AS ways,
       round(sum(ST_Length(ST_Intersection(c.g,w.line)))/1000,3) AS km_in_circle
FROM cameras c JOIN ways w ON ST_Intersects(c.g,w.line)
GROUP BY c.camera,w.highway,w.width_source ORDER BY ALL;
-- Sample equal subinterval midpoints of each clipped route fragment, never more than 20 m apart.
CREATE TEMP TABLE samples AS
WITH fragments AS (
    SELECT camera,mode,source,route_id,unnest(ST_Dump(ST_CollectionExtract(ST_Intersection(c.g,r.g),2))) AS part
    FROM cameras c JOIN route r ON ST_Intersects(c.g,r.g)
), lengths AS (
    SELECT * EXCLUDE(part),part.geom AS g,ST_Length(part.geom) AS metres FROM fragments
), intervals AS (
    SELECT *,ceil(metres/20)::BIGINT AS n FROM lengths WHERE metres>0
)
SELECT row_number() OVER () AS id,camera,mode,source,metres/n AS weight_m,
       ST_LineInterpolatePoint(g,(i+0.5)/n) AS point
FROM intervals, LATERAL range(n) members(i);
-- Baseline is the union of idealized ribbons at current renderer widths. The actual screen also has joins and occlusion.
CREATE TEMP TABLE baseline_faces AS
SELECT c.camera, ST_Buffer(o.g,o.width_m/2,6) AS g FROM cameras c JOIN original o ON ST_DWithin(c.g,o.g,10);
CREATE TEMP TABLE candidate_faces AS
SELECT c.camera, a.structure, a.layer, a.road_use, meters(a.geom) AS g
FROM cameras c JOIN actual a ON ST_Intersects(degrees(c.g),a.geom);
CREATE TEMP TABLE hits AS
SELECT DISTINCT s.id,'baseline' AS variant FROM samples s JOIN baseline_faces f ON ST_Intersects(s.point,f.g)
WHERE s.camera=f.camera
UNION ALL
SELECT DISTINCT s.id,'candidate' AS variant FROM samples s JOIN candidate_faces f ON ST_Intersects(s.point,f.g)
WHERE s.camera=f.camera AND f.structure<>'tunnel';
WITH variants AS (SELECT unnest(['baseline','candidate']) AS variant)
SELECT s.camera,s.mode,s.source,v.variant,count(*) AS samples,round(sum(s.weight_m),1) AS sampled_metres,
       round(sum(CASE WHEN h.id IS NULL THEN s.weight_m ELSE 0 END),1) AS metres_outside,
       round(100*sum(CASE WHEN h.id IS NULL THEN s.weight_m ELSE 0 END)/sum(s.weight_m),2) AS pct_outside
FROM samples s CROSS JOIN variants v LEFT JOIN hits h ON s.id=h.id AND h.variant=v.variant
GROUP BY s.camera,s.mode,s.source,v.variant ORDER BY ALL;
SELECT c.camera, count(s.id) AS route_samples FROM cameras c LEFT JOIN samples s USING(camera)
GROUP BY c.camera ORDER BY c.camera;
-- Count shared coordinates on multiple raw ways at the same mapped level. Coincidence alone is not a routing edge.
CREATE TEMP TABLE shared_vertices AS
WITH nearby AS (
    SELECT c.camera,c.g AS circle,w.osm_id,w.structure,w.layer,
           unnest(ST_Dump(ST_Points(w.line))) AS vertex
    FROM cameras c JOIN ways w ON ST_Intersects(c.g,w.line)
), points AS (
    SELECT camera,circle,osm_id,structure,layer,vertex.geom AS g FROM nearby
), shared AS (
    SELECT camera,structure,layer,g,count(DISTINCT osm_id) AS n FROM points
    WHERE ST_Covers(circle,g) GROUP BY camera,structure,layer,g HAVING count(DISTINCT osm_id)>=2
)
SELECT row_number() OVER () AS id,* FROM shared;
WITH candidate_hits AS (
    SELECT DISTINCT s.id FROM shared_vertices s JOIN candidate_faces f ON ST_Intersects(s.g,f.g)
    WHERE f.camera=s.camera AND f.structure=s.structure AND f.layer=s.layer
), baseline_hits AS (
    SELECT DISTINCT s.id FROM shared_vertices s JOIN baseline_faces f ON ST_Intersects(s.g,f.g)
    WHERE f.camera=s.camera
)
SELECT s.camera,s.structure,count(*) AS shared_coordinates,
       countif(b.id IS NULL) AS outside_baseline,countif(n.id IS NULL) AS outside_matching_candidate_level
FROM shared_vertices s LEFT JOIN candidate_hits n USING(id) LEFT JOIN baseline_hits b USING(id)
GROUP BY s.camera,s.structure ORDER BY ALL;

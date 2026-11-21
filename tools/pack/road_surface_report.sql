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
CREATE TEMP TABLE seam_edges AS
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
SELECT * FROM shared;
SELECT round(coalesce(sum(ST_Length(meters(ST_Difference(lhs,rhs))) + ST_Length(meters(ST_Difference(rhs,lhs)))),0),3) AS unmatched_seam_metres
FROM seam_edges;
CREATE TEMP TABLE cameras AS
SELECT camera, ST_Buffer(meters(ST_Point(lon,lat)),radius_m,32) AS g
FROM read_csv('data/raw/road-surface/cameras.csv',header=true,
              columns={'camera':'VARCHAR','lon':'DOUBLE','lat':'DOUBLE','radius_m':'DOUBLE'});
CREATE TEMP TABLE original AS SELECT osm_id,width_m,meters(ST_GeomFromGeoJSON(geometry)) AS g
FROM read_csv('data/raw/road-surface/baseline.csv',header=true,columns={'osm_id':'VARCHAR','width_m':'DOUBLE','geometry':'VARCHAR'});
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
SELECT row_number() OVER () AS id,camera,mode,source,route_id,metres/n AS weight_m,
       ST_LineInterpolatePoint(g,(i+0.5)/n) AS point
FROM intervals, LATERAL range(n) members(i);
-- Baseline is the union of idealized ribbons at current renderer widths. The actual screen also has joins and occlusion.
CREATE TEMP TABLE baseline_faces AS
SELECT c.camera, o.osm_id,o.width_m, ST_Buffer(o.g,o.width_m/2,6) AS g FROM cameras c JOIN original o ON ST_DWithin(c.g,o.g,10);
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

-- A seam length counts one-dimensional edge disagreement. Sample its width independently.
CREATE TEMP TABLE seam_samples AS
WITH sides AS (
    SELECT left_id AS present_tile,right_id AS opposite_tile,structure,road_use,layer,ST_Difference(lhs,rhs) AS g
    FROM seam_edges UNION ALL
    SELECT right_id,left_id,structure,road_use,layer,ST_Difference(rhs,lhs) FROM seam_edges
), fragments AS (
    SELECT * EXCLUDE(g),unnest(ST_Dump(ST_CollectionExtract(g,2))) AS part FROM sides
), counts AS (
    SELECT * EXCLUDE(part),meters(part.geom) AS geom,
           ceil(ST_Length(meters(part.geom))/0.5)::BIGINT AS n FROM fragments
)
SELECT row_number() OVER () AS id,present_tile,opposite_tile,structure,road_use,layer,
       ST_LineInterpolatePoint(geom,(i+0.5)/n) AS point, ST_Length(geom)/n AS weight_m
FROM counts, LATERAL range(n) r(i);
-- Search the opposite tile only. Separate structure, use and layer must coincide.
WITH nearby_faces AS (
    SELECT a.tile,a.structure,a.road_use,a.layer,meters(a.geom) AS g
    FROM actual a WHERE a.tile IN (SELECT opposite_tile FROM seam_samples)
), widths AS (
    SELECT s.id,s.present_tile,s.opposite_tile,s.structure,s.road_use,s.layer,s.weight_m,
           min(ST_Distance(s.point,a.g)) AS opposite_distance_m
    FROM seam_samples s LEFT JOIN nearby_faces a
      ON a.tile=s.opposite_tile AND s.structure=a.structure AND s.road_use=a.road_use AND s.layer=a.layer
      AND ST_DWithin(s.point,a.g,2)
    GROUP BY s.id,s.present_tile,s.opposite_tile,s.structure,s.road_use,s.layer,s.weight_m
)
SELECT present_tile,opposite_tile,structure,road_use,layer,
       round(sum(weight_m),3) AS seam_length_m,
       round(sum(CASE WHEN opposite_distance_m<=0.15 THEN weight_m ELSE 0 END),3) AS within_15cm_m,
       round(sum(CASE WHEN opposite_distance_m>0.15 AND opposite_distance_m<=2 THEN weight_m ELSE 0 END),3) AS between_15cm_and_2m_m,
       round(sum(CASE WHEN opposite_distance_m IS NULL THEN weight_m ELSE 0 END),3) AS beyond_2m_m
FROM widths GROUP BY ALL ORDER BY seam_length_m DESC;

-- Attribute car and bus coverage losses without changing a single road width.
CREATE TEMP TABLE missed AS
SELECT s.*,b.id IS NOT NULL AS hit_baseline
FROM samples s LEFT JOIN hits b ON b.id=s.id AND b.variant='baseline'
LEFT JOIN hits c ON c.id=s.id AND c.variant='candidate'
WHERE s.mode IN ('car','bus') AND c.id IS NULL;
CREATE TEMP TABLE tunnel_hits AS
SELECT DISTINCT s.id FROM missed s JOIN candidate_faces a ON ST_Intersects(s.point,a.g)
WHERE s.camera=a.camera AND a.structure='tunnel' AND a.road_use='road';
CREATE TEMP TABLE nearby_motor_ways AS
SELECT c.camera,w.* FROM cameras c JOIN ways w ON ST_DWithin(c.g,w.line,60) WHERE w.road_use='road';
CREATE TEMP TABLE offsets AS
SELECT s.id,w.osm_id,w.highway,w.structure,w.width_m,w.width_source,ST_Distance(s.point,w.line) AS offset_m
FROM missed s JOIN nearby_motor_ways w ON s.camera=w.camera AND ST_DWithin(s.point,w.line,60);
CREATE TEMP TABLE legacy_way_hits AS
SELECT DISTINCT s.id,b.osm_id,b.width_m AS old_width_m
FROM missed s JOIN baseline_faces b ON ST_Intersects(s.point,b.g) WHERE s.camera=b.camera;
CREATE TEMP TABLE missed_detail AS
WITH conditions AS (
    SELECT o.id,
       bool_or(structure<>'tunnel' AND offset_m<=width_m/2) AS inside_raw_width,
       bool_or(structure<>'tunnel' AND offset_m<=old_width_m/2 AND offset_m>width_m/2) AS inside_old_width,
       min(offset_m-width_m/2) FILTER (WHERE structure<>'tunnel') AS candidate_clearance_m,
       min(offset_m) FILTER (WHERE structure<>'tunnel') AS centre_offset_m
    FROM offsets o LEFT JOIN legacy_way_hits b ON o.id=b.id AND o.osm_id=b.osm_id GROUP BY o.id
), nearest AS (
    SELECT * FROM offsets WHERE structure<>'tunnel'
    QUALIFY row_number() OVER (PARTITION BY id ORDER BY offset_m,osm_id)=1
)
SELECT s.camera,s.mode,s.source,s.route_id,s.weight_m,s.hit_baseline,n.osm_id,n.highway,n.width_m,n.width_source,
       c.centre_offset_m,c.candidate_clearance_m,
       CASE WHEN t.id IS NOT NULL THEN 'covered_by_hidden_tunnel'
            WHEN c.inside_raw_width THEN 'inside_raw_width_outside_serialized_face'
            WHEN c.candidate_clearance_m<=0.15 THEN 'within_15cm_of_modeled_edge'
            WHEN c.inside_old_width THEN 'legacy_width_covers_same_raw_way'
            WHEN s.hit_baseline THEN 'legacy_path_only'
            WHEN c.centre_offset_m IS NULL THEN 'no_visible_motor_way_within_60m'
            ELSE 'outside_both_displays' END AS evidence
FROM missed s LEFT JOIN conditions c USING(id) LEFT JOIN nearest n USING(id) LEFT JOIN tunnel_hits t USING(id);
SELECT camera,mode,source,hit_baseline,evidence,count(*) AS samples,round(sum(weight_m),1) AS missed_metres,
       round(median(centre_offset_m),2) AS median_offset_to_motor_way_m
FROM missed_detail GROUP BY ALL ORDER BY camera,mode,missed_metres DESC;
COPY (
    SELECT camera,mode,source,route_id,evidence,hit_baseline,osm_id AS nearest_visible_motor_way,
           highway,width_source,width_m,round(min(centre_offset_m),2) AS min_offset_m,
           round(max(centre_offset_m),2) AS max_offset_m,round(sum(weight_m),1) AS missed_metres
    FROM missed_detail GROUP BY ALL ORDER BY camera,mode,missed_metres DESC
) TO 'data/raw/road-surface/missed_routes.csv' (HEADER);

-- Probe the raw buffered ways on each side of the disputed edge. Compare to the actual opposite tile.
CREATE TEMP TABLE probes AS
WITH normals AS (
    SELECT *, CASE WHEN opposite_tile=present_tile+1 THEN 1 WHEN opposite_tile=present_tile-1 THEN -1 ELSE 0 END AS dx,
              CASE WHEN opposite_tile=present_tile+20 THEN 1 WHEN opposite_tile=present_tile-20 THEN -1 ELSE 0 END AS dy
    FROM seam_samples
)
SELECT id,present_tile,opposite_tile,structure,road_use,layer,weight_m,d,
       ST_Translate(point,-dx*d,-dy*d) AS inside, ST_Translate(point,dx*d,dy*d) AS opposite
FROM normals CROSS JOIN (VALUES (0.15),(0.50)) p(d);
-- Select nearby source ways before buffering. The unrounded, unclipped metric buffer is the reference.
CREATE TEMP TABLE probe_ways AS
WITH near AS (
    SELECT DISTINCT w.osm_id FROM ways w JOIN probes p
      ON ST_DWithin(w.line,p.opposite,w.width_m/2+1.1)
      AND w.structure=p.structure AND w.layer=p.layer AND w.road_use=p.road_use
)
SELECT w.osm_id,w.structure,w.layer,w.road_use,ST_Buffer(w.line,w.width_m/2,6) AS g
FROM ways w SEMI JOIN near n USING(osm_id);
CREATE TEMP TABLE probe_flags AS
SELECT p.*, EXISTS (SELECT 1 FROM probe_ways w WHERE w.structure=p.structure AND w.layer=p.layer
                    AND w.road_use=p.road_use AND ST_Covers(w.g,p.inside)) AS raw_present,
            EXISTS (SELECT 1 FROM probe_ways w WHERE w.structure=p.structure AND w.layer=p.layer
                    AND w.road_use=p.road_use AND ST_Covers(w.g,p.opposite)) AS raw_opposite,
            EXISTS (SELECT 1 FROM actual a WHERE a.tile=p.opposite_tile AND a.structure=p.structure
                    AND a.layer=p.layer AND a.road_use=p.road_use AND ST_Covers(a.geom,degrees(p.opposite))) AS packed_opposite
FROM probes p;
SELECT present_tile,opposite_tile,structure,road_use,layer,d AS probe_m,raw_present,raw_opposite,packed_opposite,
       round(sum(weight_m),3) AS seam_length_m FROM probe_flags
GROUP BY ALL ORDER BY seam_length_m DESC;

-- Keep the denominators for each public bus shape. Compare only pairs present in both series.
COPY (
    SELECT s.camera,s.route_id,s.source,count(*) AS samples,round(sum(s.weight_m),1) AS sampled_m,
           round(sum(CASE WHEN h.id IS NULL THEN s.weight_m ELSE 0 END),1) AS outside_m,
           round(100*sum(CASE WHEN h.id IS NULL THEN s.weight_m ELSE 0 END)/sum(s.weight_m),2) AS outside_pct
    FROM samples s LEFT JOIN hits h ON h.id=s.id AND h.variant='candidate'
    WHERE s.mode='bus' GROUP BY ALL ORDER BY camera,route_id,source
) TO 'data/raw/road-surface/bus_shape_coverage.csv' (HEADER);

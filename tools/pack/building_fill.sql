LOAD spatial;
SET threads = 3;
SET memory_limit = '1100MB';
CREATE OR REPLACE MACRO meters(g) AS ST_Transform(g, 'EPSG:4326','EPSG:32719', always_xy := true);
CREATE OR REPLACE MACRO degrees(g) AS ST_Transform(g, 'EPSG:32719','EPSG:4326', always_xy := true);
CREATE OR REPLACE TABLE bounds AS SELECT ST_MakeEnvelope(-70.85,-33.65,-70.45,-33.30) AS geom;
CREATE OR REPLACE TABLE home AS FROM read_csv(@blocks@, all_varchar=true);
CREATE OR REPLACE TABLE jobs AS FROM read_csv(@jobs@, all_varchar=true);
CREATE OR REPLACE TABLE census AS
WITH features AS (
    SELECT MANZENT::BIGINT::VARCHAR block_id, CUT::VARCHAR cut, SHAPE::GEOMETRY geom FROM ST_Read(@cartography@, layer='Manzanas_CPV24')
    UNION ALL
    SELECT MANZENT::BIGINT::VARCHAR, CUT::VARCHAR, SHAPE::GEOMETRY FROM ST_Read(@cartography@, layer='Entidades_CPV24')
), inputs AS (
    SELECT f.block_id, f.cut, coalesce(h.population::INT,0) pop, coalesce(h.dwellings::INT,0) dwellings,
           coalesce(j.jobs::INT,0) jobs, meters(ST_MakeValid(f.geom)) geom,
           meters(ST_Intersection(ST_MakeValid(f.geom),b.geom)) AS clipped_geom
    FROM features f
    LEFT JOIN home h USING(block_id)
    LEFT JOIN jobs j USING(block_id) CROSS JOIN bounds b
    WHERE ST_Intersects(f.geom,b.geom)
)
SELECT *, ST_Area(geom) area FROM inputs;

CREATE OR REPLACE TABLE real AS
SELECT osm_id, part, height_m, levels, meters(ST_MakeValid(geom::GEOMETRY)) geom
FROM ST_Read(@components@);

CREATE OR REPLACE TABLE hits AS
SELECT r.osm_id,r.part,c.block_id, ST_Area(ST_Intersection(r.geom,c.geom)) overlap_m2,
       ST_Covers(c.geom,ST_PointOnSurface(r.geom)) contains_point
FROM real r JOIN census c ON ST_Intersects(c.geom,r.geom);

CREATE OR REPLACE TABLE assigned AS
SELECT osm_id, part, block_id FROM hits WHERE overlap_m2>0
QUALIFY row_number() OVER (PARTITION BY osm_id,part ORDER BY contains_point DESC, overlap_m2 DESC, block_id)=1;

CREATE OR REPLACE TABLE coverage AS
SELECT c.*, coalesce(h.shapes,0) real_shapes
FROM census c LEFT JOIN (SELECT block_id,count(*) shapes FROM hits GROUP BY block_id) h USING(block_id);

SELECT count(*) AS real_shapes, count(a.block_id) AS assigned_shapes FROM real r LEFT JOIN assigned a USING(osm_id,part);
SELECT count(*) AS populated_blocks, count(*) FILTER (WHERE real_shapes=0) AS empty_blocks,
       sum(dwellings) FILTER (WHERE real_shapes=0) AS empty_block_dwellings FROM coverage WHERE pop>0;

-- Keep the height proxy independent of how many buildings OSM happens to contain.
-- One floor is 3 m. Assumed floor areas are 65 m² per dwelling and 20 m² per job.
CREATE OR REPLACE TABLE height_limit AS
SELECT quantile_cont(coalesce(height_m,3.5*levels),0.99) AS cap
FROM real WHERE coalesce(height_m,levels)>0;
CREATE OR REPLACE MACRO density_height(d,j,a) AS
    round(least((SELECT cap FROM height_limit), 3 * greatest(1, (65*d + 20*j)/(0.30*a))), 1);

COPY (
   SELECT r.osm_id,r.part,c.cut,c.block_id,
          coalesce(density_height(c.dwellings,c.jobs,c.area),3.0) AS density_height_m
   FROM real r LEFT JOIN assigned a USING(osm_id,part) LEFT JOIN census c USING(block_id)
   ORDER BY r.osm_id,r.part
) TO @real_blocks@ (HEADER);

-- Learn footprint grain from well-mapped populated blocks in the same urban box.
CREATE OR REPLACE TABLE grain AS
WITH sample AS (
    SELECT c.block_id, count(*) AS n, median(ST_Area(r.geom)) AS typical_area,
           sum(ST_Area(r.geom))/c.area AS cover, c.dwellings
    FROM assigned a JOIN real r USING(osm_id,part) JOIN census c USING(block_id)
    WHERE c.pop>0 GROUP BY c.block_id,c.area,c.dwellings
)
SELECT count(*) AS reference_blocks, median(typical_area) AS footprint_m2,
       median(cover) AS developed_share, median(dwellings/n) AS dwellings_per_shape
FROM sample WHERE n>=5 AND cover BETWEEN 0.2 AND 0.9;

-- Clip at the city boundary. Keep each component separate. Set back from its edge by 2 m.
CREATE OR REPLACE TABLE sites AS
WITH clipped AS (
    SELECT c.block_id,c.cut,c.dwellings,c.jobs,c.area,
           unnest(ST_Dump(ST_CollectionExtract(c.clipped_geom,3))) AS fragment
    FROM coverage c WHERE c.pop>0 AND c.real_shapes=0
), interior AS (
    SELECT block_id,cut,dwellings,jobs,area,fragment.path AS component,
           ST_Area(fragment.geom) AS clipped_m2, ST_Buffer(fragment.geom,-2) AS geom
    FROM clipped
)
SELECT s.*, greatest(1, round(least(clipped_m2*g.developed_share/g.footprint_m2,
             clipped_m2/area*dwellings/g.dwellings_per_shape)))::INT AS target,
       g.footprint_m2
FROM interior s, grain g WHERE NOT ST_IsEmpty(geom) AND ST_Area(geom)>=12;

-- Align a regular grid with the long side of the local block. Use the full geometry
-- to accept footprints, including courtyards, concave edges and irregular block ends.
CREATE OR REPLACE TABLE grids AS
WITH edges AS (
    SELECT *, ST_ExteriorRing(ST_MinimumRotatedRectangle(geom)) AS ring FROM sites
), angles AS (
    SELECT *, atan2(ST_Y(ST_PointN(ring,2))-ST_Y(ST_PointN(ring,1)),
                    ST_X(ST_PointN(ring,2))-ST_X(ST_PointN(ring,1))) AS angle FROM edges
), rotated AS (
    SELECT * EXCLUDE(ring), ST_Extent(ST_Rotate(geom,-angle)) box FROM angles
), extents AS (
    SELECT *, ST_XMax(box)-ST_XMin(box) AS width, ST_YMax(box)-ST_YMin(box) AS depth,
           sqrt(ST_Area(geom)/target) AS spacing FROM rotated
)
SELECT *, greatest(1,ceil(width/spacing)::INT) AS nx,
          greatest(1,ceil(depth/spacing)::INT) AS ny FROM extents;

CREATE OR REPLACE MACRO factor(seed, base, steps) AS (base + (seed % steps)::INT/100.0);

CREATE OR REPLACE TABLE candidates AS
WITH cells AS (
    SELECT g.*,x,y,width/nx AS dx,depth/ny AS dy,
           md5_number_lower(concat_ws('/',block_id,component,x,y)) AS seed FROM grids g,
           LATERAL range(nx) xs(x), LATERAL range(ny) ys(y)
), dimensions AS (
    SELECT *, least(0.76*dx, sqrt(footprint_m2)*factor(seed,0.7,39)) AS w,
              least(0.76*dy, sqrt(footprint_m2)*factor(seed//41,0.85,55)) AS h,
              ST_XMin(box) + (x+factor(seed//2417,0.45,11))*dx AS cx,
              ST_YMin(box) + (y+factor(seed//5107,0.45,11))*dy AS cy FROM cells
), rectangles AS (
    SELECT block_id,component,target,cut,geom,seed,
           ST_Rotate(ST_MakeEnvelope(cx-w/2,cy-h/2,cx+w/2,cy+h/2),angle) AS footprint
    FROM dimensions
)
SELECT block_id,component,cut,seed,footprint FROM rectangles
WHERE ST_Covers(geom,footprint)
QUALIFY row_number() OVER (PARTITION BY block_id,component ORDER BY seed) <= target;

-- Thin or irregular blocks can have no full rectangle. Their inscribed circle
-- locates one small outline wholly inside the block. It uses the same edge setback.
CREATE OR REPLACE TABLE generated AS
WITH missing AS (
    SELECT g.*,ST_MaximumInscribedCircle(g.geom,1.0) AS circle
    FROM grids g ANTI JOIN candidates c USING(block_id,component)
), fallback AS (
    SELECT block_id,component,cut,md5_number_lower(concat_ws('/',block_id,component,'inset')) AS seed,angle,circle,
           least(circle.radius*0.6,sqrt(footprint_m2)/2) AS radius FROM missing
), footprints AS (
    SELECT block_id,component,cut,seed,footprint FROM candidates
    UNION ALL
    SELECT block_id,component,cut,seed,
           ST_Translate(ST_Rotate(ST_MakeEnvelope(-radius,-radius,radius,radius),angle),
                        ST_X(circle.center),ST_Y(circle.center)) AS footprint FROM fallback WHERE radius>=1
), rounded AS (
    SELECT f.block_id,f.cut,f.seed, ST_ReducePrecision(degrees(footprint),0.00001) AS polygon,
           round(least((SELECT cap FROM height_limit),greatest(3.0,density_height(c.dwellings,c.jobs,c.area)*factor(seed//10467,0.78,46))),1) AS height_m
    FROM footprints f JOIN census c USING(block_id)
)
SELECT rounded.* FROM rounded,bounds WHERE ST_Area(polygon)>0 AND ST_Covers(bounds.geom,polygon);

COPY (
    SELECT json_object('type','Feature','geometry',ST_AsGeoJSON(polygon)::JSON,
           'properties',json_object('block_id',block_id,'cut',cut,'source','generated','height_m',height_m))::VARCHAR AS feature
    FROM generated ORDER BY block_id,seed
) TO @generated@ (FORMAT CSV,HEADER false,DELIMITER '\t',QUOTE '',ESCAPE '');

COPY (
    WITH by_block AS (SELECT block_id,count(*) n FROM generated GROUP BY block_id),
         mapped AS (SELECT c.cut,count(*) n FROM assigned a JOIN census c USING(block_id) GROUP BY c.cut),
         counts AS (
             SELECT c.cut,count(*) AS populated_blocks,
                    count(*) FILTER (WHERE c.real_shapes>0) AS with_real_osm,
                    count(*) FILTER (WHERE c.real_shapes=0) AS without_real_osm,
                    count(*) FILTER (WHERE g.n>0) AS with_generated,
                    coalesce(sum(g.n),0)::BIGINT AS generated_shapes
             FROM coverage c LEFT JOIN by_block g USING(block_id)
             WHERE c.pop>0 GROUP BY c.cut
         )
    SELECT cuts.cut, coalesce(populated_blocks,0) AS populated_blocks,
           coalesce(with_real_osm,0) AS with_real_osm, coalesce(without_real_osm,0) AS without_real_osm,
           coalesce(with_generated,0) AS with_generated, coalesce(generated_shapes,0) AS generated_shapes,
           coalesce(mapped.n,0) AS assigned_real_shapes,
           round(counts.with_real_osm::DOUBLE/nullif(counts.populated_blocks,0),6) AS real_share
    FROM (SELECT DISTINCT cut FROM home) cuts
    LEFT JOIN counts USING(cut) LEFT JOIN mapped USING(cut) ORDER BY cuts.cut
) TO @coverage@ (HEADER);

SELECT * FROM grain;
SELECT count(*) AS generated_shapes,count(DISTINCT block_id) AS generated_blocks FROM generated;

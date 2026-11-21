LOAD spatial;
SET threads=3;
SET memory_limit='2GB';
CREATE OR REPLACE MACRO meters(g) AS ST_Transform(g,'EPSG:4326','EPSG:32719',always_xy:=true);
CREATE OR REPLACE MACRO degrees(g) AS ST_Transform(g,'EPSG:32719','EPSG:4326',always_xy:=true);
CREATE OR REPLACE TABLE ways AS
SELECT osm_id, highway, structure, layer, road_use, width_m, width_source, meters(ST_GeomFromGeoJSON(geometry)) AS line
FROM read_csv('data/raw/road-surface/ways.csv',header=true,
              columns={'osm_id':'VARCHAR','highway':'VARCHAR','structure':'VARCHAR','layer':'INTEGER',
                       'road_use':'VARCHAR','width_m':'DOUBLE','width_source':'VARCHAR','geometry':'VARCHAR'});
-- Use the building grid's integer coordinates. Its northern row is only 0.01 degrees tall.
CREATE OR REPLACE TABLE tiles AS
SELECT x::INT AS col, y::INT AS row_id, (y*20+x)::INT AS tile,
       ST_MakeEnvelope((-7085000+2000*x)/100000.0,(-3365000+2000*y)/100000.0,
                       (-7085000+2000*(x+1))/100000.0,least(-3330000,-3365000+2000*(y+1))/100000.0) AS geom
FROM range(20) xs(x), range(18) ys(y);
-- GEOS round caps and round joins. Shared precision precedes both clipping and union.
CREATE OR REPLACE TEMP TABLE ribbons AS
SELECT osm_id,structure,layer,road_use,
       ST_ReducePrecision(degrees(ST_Buffer(line,width_m/2,6)),0.000001) AS geom
FROM ways;
CREATE OR REPLACE TABLE faces AS
WITH clipped AS (
    SELECT t.tile,t.col,t.row_id,r.osm_id,r.structure,r.layer,r.road_use,
           ST_ReducePrecision(ST_Intersection(r.geom,t.geom),0.000001) AS geom
    FROM ribbons r JOIN tiles t ON ST_Intersects(r.geom,t.geom)
), merged AS (
    SELECT tile,col,row_id,structure,layer,road_use,
           ST_Union_Agg(ST_CollectionExtract(geom,3) ORDER BY osm_id) AS geom
    FROM clipped GROUP BY tile,col,row_id,structure,layer,road_use
), parts AS (
    SELECT * EXCLUDE(geom),unnest(ST_Dump(ST_CollectionExtract(geom,3))) AS part FROM merged
)
SELECT * EXCLUDE(part), part.geom AS geom FROM parts WHERE ST_Area(part.geom)>0;
COPY (
    SELECT json_object('type','Feature','properties',json_object('tile',tile,'column',col,'row',row_id,
           'layer',layer,'structure',structure,'kind',CASE WHEN road_use='sidewalk' THEN 'sidewalk' ELSE structure END,
           'use',road_use),'geometry',ST_AsGeoJSON(geom)::JSON)::VARCHAR
    FROM faces ORDER BY tile,structure,layer,road_use,ST_AsWKB(geom)
) TO 'data/raw/road-surface/features.tsv' (FORMAT CSV,HEADER false,QUOTE '',ESCAPE '',DELIMITER '\t');
SELECT highway,width_source,count(*) AS source_ways,round(sum(ST_Length(line))/1000,2) AS source_km
FROM ways GROUP BY highway,width_source ORDER BY highway,width_source;
SELECT structure,layer,road_use,count(*) AS pieces, sum(ST_NumInteriorRings(geom)) AS holes
FROM faces GROUP BY structure,layer,road_use ORDER BY structure,layer,road_use;

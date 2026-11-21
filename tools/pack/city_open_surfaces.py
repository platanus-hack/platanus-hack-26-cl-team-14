"""Pack mapped open-air parking, development sites, pitches and track areas."""
from city_ground_layers import RAW, build

FILTERS = ("wr/amenity=parking", "wr/landuse=vacant,brownfield,greenfield", "wr/leisure=pitch,track")
KIND = """CASE
  WHEN area='no' THEN NULL
  WHEN amenity='parking' AND (parking='surface' OR parking IS NULL)
       AND coalesce(covered,'no')='no' AND coalesce(indoor,'no')='no'
       AND coalesce(building,'no')='no' AND coalesce("building:part",'no')='no'
       AND coalesce(try_cast(layer AS INTEGER),0)>=0
    THEN CASE WHEN parking='surface' THEN 'surface_parking' ELSE 'parking_unspecified' END
  WHEN landuse IN ('vacant','brownfield','greenfield') THEN landuse
  WHEN leisure='pitch' AND coalesce(indoor,'no')='no' THEN 'pitch'
  WHEN leisure='track' AND coalesce(indoor,'no')='no'
       AND (area='yes' OR cast(substr(id,2) AS BIGINT)%2=1) THEN 'track' END"""

if __name__ == "__main__":
    RAW.mkdir(parents=True, exist_ok=True)
    build("open_surfaces", FILTERS, KIND, True)

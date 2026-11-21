-- Start with OSRM bicycle access, directional cycle lanes, barriers, and turn rules.
local profile_dir = os.getenv('OSRM_PROFILES')
package.path = profile_dir .. '/?.lua;' .. package.path
local bike = dofile(profile_dir .. '/bicycle.lua')
local base_setup, base_way = bike.setup, bike.process_way

function bike.setup()
  local p = base_setup()
  p.properties.weight_name = 'cyclability'
  p.use_public_transport = false
  for _, key in ipairs({'customers', 'permit', 'destination', 'dismount'}) do
    p.access_tag_blacklist[key] = true
  end
  return p
end

function bike.process_way(p, way, result, relations)
  local highway = way:get_value_by_key('highway')
  if not highway or highway == '' or highway == 'motorway' or highway == 'motorway_link'
      or highway == 'trunk' or highway == 'trunk_link' or highway == 'steps'
      or way:get_value_by_key('motorroad') == 'yes' then return end
  local bicycle = way:get_value_by_key('bicycle')
  if (highway == 'footway' or highway == 'pedestrian' or highway == 'path')
      and not p.access_tag_whitelist[bicycle] then return end
  base_way(p, way, result, relations)
  if result.forward_mode ~= mode.cycling then result.forward_mode = mode.inaccessible end
  if result.backward_mode ~= mode.cycling then result.backward_mode = mode.inaccessible end
end

return bike

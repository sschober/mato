#!/usr/bin/env lua
if( #arg < 1 ) then
  error("expected file name as first arg")
end

-- crazy, we have to implement it
function file_exists(name)
  local f <close> = io.open(name, "r")
  return f ~= nil
end

-- creating the file by opening it for writing
function create_empty_file(name)
  local f <close> = io.open(name, "w")
end

function wt_cli(cmd)
  return io.popen("wezterm cli " .. cmd):read("*a"):gsub("[\n]","")
end

file_to_open=arg[1]

if( not file_exists( file_to_open )) then
  create_empty_file(file_to_open)
end

file_name = file_to_open:gsub("([^.]*)%..*", "%1") .. ".pdf"
print("file_name: " .. file_name)

local origin_pane_id = os.getenv("WEZTERM_PANE")

local micro_pane_id = wt_cli("spawn zsh -c \"micro " 
      .. file_to_open .. "\"")
print("micro_pane_id: " .. micro_pane_id)

local matopdf_cmd = "$HOME/.cargo/bin/matopdf -w -v "
local matopdf_pane_id = wt_cli( "split-pane --pane-id " 
  .. micro_pane_id
  .. " --percent 10 --bottom zsh -c \""
  .. matopdf_cmd 
  .. file_to_open .. "\"" )
print( "matopdf_pane_id: " .. matopdf_pane_id)


-- we need to wait for matopdf to finish, otherwise
-- termpdf.py bails out, because the pdf file does
-- not exist
os.execute( "sleep 1" )

local termpdf_cmd = "$HOME/bin/termpdf.py " 
local termpdf_pane_id = wt_cli("split-pane --pane-id " .. micro_pane_id
    .. " --top-level --right zsh -c \""
    .. termpdf_cmd 
    .. file_name 
    .. "\"")

wt_cli("activate-pane --pane-id " .. micro_pane_id)

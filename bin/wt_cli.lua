local M = {}

function M.wt_cli(cmd)
  local cli_cmd = "wezterm cli " .. cmd
  print(cli_cmd)
  return io.popen(cli_cmd):read("*a"):gsub("[\n]","")
end

function M.spawn(cmd)
  local id = M.wt_cli("spawn "..M.zsh_c(cmd))
  print("=> spawned new pane with id: "..id)
  return id
end
  
function M.split_pane_id(id, opts, cmd)
  local id = M.wt_cli("split-pane --pane-id " .. id ..opts.. M.zsh_c(cmd))
  print("=> split new pane with id: "..id)
  return id
end

function M.zsh_c(cmd)
  return "zsh -c \"" .. cmd .. "\""
end

function M.activate_pane(id)
  M.wt_cli("activate-pane --pane-id " .. id)
end

return M

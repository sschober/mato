local M = {}

function M.wt_cli(cmd)
  return io.popen("wezterm cli " .. cmd):read("*a"):gsub("[\n]","")
end

function M.spawn(cmd)
  return M.wt_cli("spawn "..M.zsh_c(cmd))
end
  
function M.split_pane_id(id, opts, cmd)
  return M.wt_cli("split-pane --pane-id " .. id ..opts.. M.zsh_c(cmd))
end

function M.zsh_c(cmd)
  return "zsh -c \"" .. cmd .. "\""
end

function M.activate_pane(id)
  M.wt_cli("activate-pane --pane-id " .. id)
end

return M

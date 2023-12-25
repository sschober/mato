local M = {}

-- crazy, we have to implement it
function M.file_exists(name)
  local f <close> = io.open(name, "r")
  return f ~= nil
end

-- creating the file by opening it for writing
function M.create_empty_file(name)
  local f <close> = io.open(name, "w")
end

function M.create_if_not_exists(name)
  if not M.file_exists(name) then
    M.create_empty_file(name)
  end
end

function M.replace_extension(file, extension)
  return file:gsub("([^.]*)%..*", "%1") .. extension
end

return M

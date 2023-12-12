local utils = {}

function utils.file_path()
    return debug.getinfo(2, 'S').source:sub(2)
end

local root_path = vim.fn.fnamemodify(utils.file_path(), ':h:h:h')
local dll_suffix = package.config:sub(1, 1) == '/' and '.so' or '.dll'

---@param name string
function utils.loadlib(name)
    local path = root_path .. '/bin/' .. name .. dll_suffix

    local dash = name:find('-', 1, true)
    local modname = dash and name:sub(dash + 1) or name
    local f, _ = package.loadlib(path, 'luaopen_' .. modname:gsub('%.', '_'))

    local mod = f()

    -- package.loaded[name] = mod
    return mod
end

return utils

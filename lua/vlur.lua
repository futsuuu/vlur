local utils = {}
do
    local function get_root_path()
            local file = debug.getinfo(2, 'S').source:sub(2)
        return vim.fn.fnamemodify(file, ':p:h:h')
    end

    local root_path = get_root_path()
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
end

---@class vlur
local M = {}

local lib = utils.loadlib 'vlur'

if lib.debug then
    M._lib = lib
end

return M

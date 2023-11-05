local nvim = {}
do
    -- NOTE: For compatibility and maintainability, use vim.* as little as possible outside of this scope.
    local api = vim.api

    ---@param name string
    ---@return any
    nvim.get_opt = function(name)
        return api.nvim_get_option_value(name, {})
    end

    ---@param name string
    ---@param value any
    nvim.set_opt = function(name, value)
        api.nvim_set_option_value(name, value, {})
    end

    ---@param command string
    nvim.exec = function(command)
        api.nvim_exec2(command, {})
    end
end

local utils = {}
do
    local function get_root_path()
        local file = debug.getinfo(2, 'S').source:sub(2)
        return vim.fn.fnamemodify(file, ':p:h:h')
    end

    local root_path = get_root_path()
    local dll_suffix = package.config:sub(1, 1) == '/' and '.so' or '.dll'

    ---@param name string
    utils.loadlib = function(name)
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

M.setup = function()
    local lib = utils.loadlib 'vlur'

    if lib.debug then
        M._lib = lib
    end

    lib.setup(nvim)
end

return M

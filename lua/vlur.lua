local root_path = (function()
    local file = debug.getinfo(2, 'S').source:sub(2)
    return vim.fn.fnamemodify(file, ':h:h')
end)()

local lib = (function()
    local libname
    if package.config:sub(1, 1) == '/' then
        libname = root_path .. [[/bin/vlur.so]]
    else
        libname = root_path .. [[\bin\vlur.dll]]
    end
    local f, _ = package.loadlib(libname, 'luaopen_vlur')
    local mod = f()
    return mod
end)()

---@class vlur
local M = {}
M.lazy = lib.lazy
M.install = lib.install
if lib.debug then
    M._lib = lib
end

function M.setup(plugins, config)
    lib.setup(plugins or {}, config or {})
end

return M

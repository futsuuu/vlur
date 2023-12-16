local utils = require 'vlur.utils'
local lib = utils.loadlib 'vlur'

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

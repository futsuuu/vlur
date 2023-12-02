local utils = require 'vlur.utils'
local lib = utils.loadlib 'vlur'

---@class vlur
local M = {}
M.lazy = lib.lazy
if lib.debug then
    M._lib = lib
end

function M.setup(plugins, config)
    lib.setup {
        plugins = plugins or {},
        config = config or {},
    }
end

return M

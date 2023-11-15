local nvim = require 'vlur.nvim'
local utils = require 'vlur.utils'

---@class vlur
local M = {}

function M.setup(plugins, config)
    local lib = utils.loadlib 'vlur'
    if lib.debug then
        M._lib = lib
    end

    local args = {}
    args.nvim = nvim
    args.plugins = plugins or {}
    args.config = config or {}

    lib.setup(args)
end

return M

local nvim = require 'vlur.nvim'
local utils = require 'vlur.utils'

---@class vlur
local M = {}

function M.setup()
    local lib = utils.loadlib 'vlur'

    if lib.debug then
        M._lib = lib
    end

    lib.setup(nvim)
end

return M

local window = require 'vlur.ui.window'

local M = {}

function M.open()
    window:open():update()
end

function M.update()
    window:update()
end

function M.hide()
    window:hide()
end

return M

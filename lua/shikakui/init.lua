local api = vim.api

local M = {
    hooks = require 'shikakui.hooks',
    element = require 'shikakui.element',
}

---@alias shikakui.Primitive
---| string
---| number
---| boolean
---| nil

---@alias shikakui.Node
---| shikakui.Element
---| shikakui.Primitive

---@param window window
---@param node shikakui.Node
function M.render(window, node)
    M.element
        .from_node(node)
        :render(api.nvim_win_get_buf(window), { line = 1, col = 1 }, {
            width = api.nvim_win_get_width(window),
            height = api.nvim_win_get_height(window),
        })
end

---@generic T: fun(...): shikakui.Node
---@param func T
---@return T
function M.component(func)
    return require('shikakui.component').new(func)
end

return M

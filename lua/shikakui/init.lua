local utils = require 'shikakui.utils'

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
---| shikakui.ElementBuilder
---| shikakui.Primitive

---@param node shikakui.Node
function M.render(node)
    local root = M.element.root(nil, node)
    root:build():render {
        pos = utils.Pos(),
        size = utils.Size(vim.o.columns, vim.o.lines),
    }
end

---@generic T: fun(...): shikakui.Node
---@param func T
---@return T
function M.component(func)
    return require('shikakui.component').new(func)
end

return M

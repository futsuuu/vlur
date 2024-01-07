local Element = require 'shikakui.element.base'

---@class shikakui.element.Highlight: shikakui.Element
---@field hl_group string
---@field child shikakui.Element
local M = setmetatable({}, { __index = Element })

---@param hl_group string
---@param child shikakui.Element
---@return shikakui.element.Highlight
function M.new(hl_group, child)
    local self = setmetatable({}, { __index = M })
    self.hl_group = hl_group
    self.child = child
    return self
end

---@param buffer buffer
---@param pos shikakui.Position
---@param size shikakui.Size
---@return shikakui.Size
function M:render(buffer, pos, size)
    local rendered_size = self.child:render(buffer, pos, size)
    vim.api.nvim_buf_add_highlight(
        buffer,
        0,
        self.hl_group,
        pos.line - 1,
        pos.col - 1,
        pos.col - 1 + rendered_size.width
    )
    return rendered_size
end

return M

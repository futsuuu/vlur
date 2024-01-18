local Element = require 'shikakui.element.base'

---@class shikakui.element.Highlight: shikakui.Element
---@field hl_group string
---@field child shikakui.Element
local M = setmetatable({}, { __index = Element })

---@return shikakui.element.Highlight
function M.new()
    return setmetatable({}, { __index = M })
end

---@param hl_group string
---@param parent shikakui.Element
---@param child shikakui.Element
function M:init(hl_group, parent, child)
    self.hl_group = hl_group
    self.parent = parent
    self.child = child
end

function M:height_range()
    return self.child:height_range()
end

function M:width_range()
    return self.child:width_range()
end

---@param area shikakui.Area
---@return shikakui.Area
function M:render(area)
    assert(area.buf, 'buffer not found')
    local r_area = self.child:render(area)
    for y = r_area.pos.y - 1, r_area.pos.y + r_area.size.height - 1 do
        vim.api.nvim_buf_add_highlight(
            area.buf,
            0,
            self.hl_group,
            y,
            r_area.pos.x - 1,
            r_area.pos.x - 1 + r_area.size.width
        )
    end
    return r_area
end

return M

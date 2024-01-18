local Element = require 'shikakui.element.base'
local utils = require 'shikakui.utils'

---@class shikakui.element.Text: shikakui.Element
---@field text string
local M = setmetatable({}, { __index = Element })

---@return shikakui.element.Text
function M.new()
    return setmetatable({}, { __index = M })
end

---@param text shikakui.Primitive
---@param parent shikakui.Element
function M:init(text, parent)
    self.text = tostring(text)
    self.parent = parent
end

---@return shikakui.Range
function M:width_range()
    return utils.Range(self.text:len())
end

---@return shikakui.Range
function M:height_range()
    return utils.Range(1)
end

---@param area shikakui.Area
---@return shikakui.Area
function M:render(area)
    assert(area.buf, 'buffer not found')
    utils.set_text(area.buf, area.pos, self.text)
    return {
        win = area.win,
        buf = area.buf,
        pos = area.pos,
        size = utils.Size(self.text:len(), 1),
    }
end

return M

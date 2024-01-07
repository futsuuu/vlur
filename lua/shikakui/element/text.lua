local Element = require 'shikakui.element.base'
local utils = require 'shikakui.utils'

---@class shikakui.element.Text: shikakui.Element
---@field text string
local M = setmetatable({}, { __index = Element })

---@param text shikakui.Primitive
---@return shikakui.element.Text
function M.new(text)
    local self = setmetatable({}, { __index = M })
    self.text = tostring(text)
    return self
end

---@return shikakui.Size
function M:get_min_size()
    return {
        width = self.text:len(),
        height = 1,
    }
end

---@param buffer buffer
---@param pos shikakui.Position
---@param _size shikakui.Size
function M:render(buffer, pos, _size)
    utils.set_text(buffer, pos, self.text)
    return self:get_min_size()
end

return M

local Element = require 'shikakui.element.base'

---@class shikakui.element.Root: shikakui.Element
local M = setmetatable({}, { __index = Element })

---@return shikakui.element.Root
function M.new()
    return setmetatable({}, { __index = M })
end

---@param child shikakui.Element
function M:init(_, _, child)
    self.child = child
end

function M:height_range()
    return self.child:height_range()
end

function M:width_range()
    return self.child:width_range()
end

function M:render(area)
    return self.child:render(area)
end

return M

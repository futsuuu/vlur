---@class shikakui.Element
local M = {}

---@diagnostic disable: unused-local, missing-return

---@package
function M.new() end

---@package
---@return shikakui.Size
function M:get_min_size()
    ---@diagnostic disable-next-line: undefined-field
    return self.child:get_min_size()
end

---@package
---@param buffer buffer
---@param pos shikakui.Position
---@param size shikakui.Size
---@return shikakui.Size
function M:render(buffer, pos, size) end

---@diagnostic enable

return M

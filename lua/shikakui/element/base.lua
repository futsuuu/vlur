---@class shikakui.Element
local M = {}

---@diagnostic disable: unused-local, missing-return

---@package
---@return shikakui.Element
function M.new() end

---@param opts any
---@param parent shikakui.Element
---@param ... shikakui.Element
function M:init(opts, parent, ...) end

---@package
---@return shikakui.Range
function M:width_range() end

---@package
---@return shikakui.Range
function M:height_range() end

---@package
---@param area shikakui.Area
---@return shikakui.Area
function M:render(area) end

---@diagnostic enable

return M

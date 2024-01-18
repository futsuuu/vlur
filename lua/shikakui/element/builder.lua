---@class shikakui.ElementBuilder
---@field private new_element shikakui.Element
---@field private opts any
---@field private children shikakui.Element[]
local M = {}

---@package
---@param new_element shikakui.Element
---@param opts any
---@param ... shikakui.ElementBuilder
---@return shikakui.ElementBuilder
function M.new(new_element, opts, ...)
    ---@type shikakui.Element[]
    local children = {}
    for _, child in ipairs { ... } do
        table.insert(children, child:build(new_element))
    end
    local self = setmetatable({}, { __index = M })
    self.new_element = new_element
    self.opts = opts
    self.children = children
    return self
end

---@param parent? shikakui.Element
---@return shikakui.Element
function M:build(parent)
    ---@diagnostic disable-next-line: empty-block
    if not parent then
        ---@cast parent shikakui.element.Root
    end
    self.new_element:init(self.opts, parent, unpack(self.children))
    return self.new_element
end

return M

local M = {}

---@package
---@param node shikakui.Node
---@return shikakui.Element
local function from(node)
    if type(node) == 'table' then
        return node
    end
    ---@cast node shikakui.Primitive
    return require('shikakui.element.text').new(node)
end

M.from_node = from

---@param hl_group string
---@param child shikakui.Node
function M.hl(hl_group, child)
    return require('shikakui.element.highlight').new(hl_group, from(child))
end

return M

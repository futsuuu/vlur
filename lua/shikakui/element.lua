local Builder = require 'shikakui.element.builder'

local M = {}

---@param ... shikakui.Node
---@return shikakui.ElementBuilder ...
local function to_builder(...)
    local builders = {}
    for _, node in ipairs { ... } do
        local builder
        if type(node) == 'table' then
            builder = node
        else
            ---@cast node -shikakui.ElementBuilder
            builder = M.text(node)
        end
        table.insert(builders, builder)
    end
    return unpack(builders)
end

---@private
M.from_node = to_builder

---@private
---@param child shikakui.Node
---@return shikakui.ElementBuilder
function M.root(_, child)
    local element = require('shikakui.element.root').new()
    return Builder.new(element, _, to_builder(child))
end

---@private
---@param text shikakui.Primitive
---@return shikakui.ElementBuilder
function M.text(text)
    local element = require('shikakui.element.text').new()
    return Builder.new(element, text)
end

---@param hl_group string
---@param child shikakui.Node
---@return shikakui.ElementBuilder
function M.highlight(hl_group, child)
    local element = require('shikakui.element.highlight').new()
    return Builder.new(element, hl_group, to_builder(child))
end

---@param opts shikakui.element.FloatwinOpts
---@param child shikakui.Node
---@return shikakui.ElementBuilder
function M.floatwin(opts, child)
    local element = require('shikakui.element.floatwin').new()
    return Builder.new(element, opts, to_builder(child))
end

---@param opts shikakui.element.PaddingOpts
---@param child shikakui.Node
---@return shikakui.ElementBuilder
function M.padding(opts, child)
    local element = require('shikakui.element.padding').new()
    return Builder.new(element, opts, to_builder(child))
end

return M

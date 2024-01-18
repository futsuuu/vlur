local element = require 'shikakui.element'

local M = {}

---@class shikakui.Component
---@field states { current: any, new: any }[]
---@field state_idx integer
---@field child_fn fun(...): shikakui.Node
---@field args unknown[]
---@field child shikakui.ElementBuilder
local Component = {}

---@package
---@type shikakui.Component[]
M.rendering = {}

---@generic T: fun(...): shikakui.Node
---@param func T
---@return T
function M.new(func)
    local self = setmetatable({}, {
        __index = Component,
        __call = Component.get_element,
    })
    self.child_fn = func
    self.states = {}
    return self
end

---@package
---@return shikakui.ElementBuilder
function Component:get_element(...)
    local args = { ... }
    local len = #args
    if self.args and #self.args == len then
        if len == 0 then
            return self.child
        end
        for i, arg in ipairs(args) do
            if not self.args or self.args[i] ~= arg then
                break
            elseif i == len then
                return self.child
            end
        end
    end
    self.args = args
    self.state_idx = 0
    table.insert(M.rendering, 1, self)
    self.child = element.from_node(self.child_fn(...))
    table.remove(M.rendering, 1)
    return self.child
end

---@package
---@generic T
---@param initial T
---@return T, fun(new: T)
function Component:use_state(initial)
    self.state_idx = self.state_idx + 1
    local i = self.state_idx

    if not self.states[i] then
        self.states[i] = {}
        self.states[i].current = initial
    elseif self.states[i].new then
        self.states[i].current = self.states[i].new
        self.states[i].new = nil
    end

    local function set_state(new)
        self.states[i].new = new
    end

    return self.states[i].current, set_state
end

return M

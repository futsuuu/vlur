local Element = require 'shikakui.element.base'
local utils = require 'shikakui.utils'

---@class shikakui.element.Padding: shikakui.Element
---@field opts { right: number, left: number, top: number, bottom: number }
local M = setmetatable({}, { __index = Element })

function M.new()
    return setmetatable({}, { __index = M })
end

---@class shikakui.element.PaddingOpts
---@field all? number
---@field horizontal? number
---@field vertical? number
---@field right? number
---@field left? number
---@field top? number
---@field bottom? number

---@param opts shikakui.element.PaddingOpts
---@param parent shikakui.Element
---@param child shikakui.Element
function M:init(opts, parent, child)
    -- stylua: ignore
    self.opts = {
        right  = opts.right  or opts.horizontal or opts.all or 0,
        left   = opts.left   or opts.horizontal or opts.all or 0,
        top    = opts.top    or opts.vertical   or opts.all or 0,
        bottom = opts.bottom or opts.vertical   or opts.all or 0,
    }
    self.parent = parent
    self.child = child
end

---@return shikakui.Range
function M:width_range()
    local child = self.child:width_range()
    local static = 0
    local dynamic = 0.0
    for _, n in ipairs { self.opts.right, self.opts.left } do
        if utils.num.type(n) == 'integer' then
            static = static + n
        else
            dynamic = dynamic + n
        end
    end

    if dynamic == 0 then
        return utils.Range(child.min + static, child.max + static)
    end

    static = static + child.min
    if dynamic > 1 then
        return utils.Range(static, math.huge)
    end

    --                 width
    -- |---------------------|--------------|
    --         static         dynamic * width
    --
    -- 0 <= dynamic < 1
    -- width : dynamic * width = 1 : dynamic
    --
    -- static : dynamic * width = 1 - dynamic : dynamic
    -- dynamic * width = (static * dynamic) / (1 - dynamic)
    -- width = (static * dynamic) / (1 - dynamic) + static
    return utils.Range((static * dynamic) / (1 - dynamic) + static, math.huge)
end

---@return shikakui.Range
function M:height_range()
    local child = self.child:height_range()
    local static = 0
    local dynamic = 0.0
    for _, n in ipairs { self.opts.top, self.opts.bottom } do
        if utils.num.type(n) == 'integer' then
            static = static + n
        else
            dynamic = dynamic + n
        end
    end
    if dynamic == 0 then
        return utils.Range(child.min + static, child.max + static)
    end
    static = static + child.min
    if dynamic > 1 then
        return utils.Range(static, math.huge)
    end
    return utils.Range((static * dynamic) / (1 - dynamic) + static, math.huge)
end

---@param area shikakui.Area
---@return shikakui.Area
function M:render(area)
    local round = utils.num.round

    ---@type { right: integer, left: integer, top: integer, bottom: integer }
    local opts = {}
    for key, n in pairs(self.opts) do
        if utils.num.type(n) == 'integer' then
            opts[key] = n
        elseif key == 'right' or key == 'left' then
            opts[key] = round(n * area.size.width)
        else
            opts[key] = round(n * area.size.height)
        end
    end

    area = vim.deepcopy(area)

    local min_width = self.child:width_range().min
    local right_left = opts.right + opts.left
    local width_rate = (area.size.width - min_width) / right_left

    width_rate = utils.num.clamp(0, width_rate, 1)
    area.pos:add_x(round(opts.left * width_rate))
    area.size:add_width(round(-right_left * width_rate))

    local min_height = self.child:height_range().min
    local top_bottom = opts.top + opts.bottom
    local height_rate = (area.size.height - min_height) / top_bottom

    height_rate = utils.num.clamp(0, height_rate, 1)
    area.pos:add_y(round(opts.top * height_rate))
    area.size:add_height(round(-top_bottom * height_rate))

    self.child:render(area)
    return area
end

return M

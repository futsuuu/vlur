local Element = require 'shikakui.element.base'
local utils = require 'shikakui.utils'

---@class shikakui.element.Padding: shikakui.Element
---@field opts shikakui.element.PaddingOpts
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
local default_opts = {
    right = 0,
    left = 0,
    top = 0,
    bottom = 0,
}

---@param opts shikakui.element.PaddingOpts
---@param parent shikakui.Element
---@param child shikakui.Element
function M:init(opts, parent, child)
    self.opts = vim.tbl_extend('keep', opts, {
        right = opts.horizontal,
        left = opts.horizontal,
        top = opts.vertical,
        bottom = opts.vertical,
    }, {
        right = opts.all,
        left = opts.all,
        top = opts.all,
        bottom = opts.all,
    }, default_opts)
    self.parent = parent
    self.child = child
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

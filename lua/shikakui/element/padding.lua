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
    ---@type shikakui.element.PaddingOpts
    local opts = {}
    for key, n in pairs(self.opts) do
        if utils.number_type(n) == 'integer' then
            opts[key] = n
        else
            if key == 'right' or key == 'left' then
                opts[key] = math.floor(area.size.width * n)
            else
                opts[key] = math.floor(area.size.height * n)
            end
        end
    end
    area = vim.deepcopy(area)
    area.pos:set_x(opts.left):set_y(opts.top)
    area.size:add_width(-opts.right - opts.left):add_height(-opts.top - opts.bottom)
    self.child:render(area)
    return area
end

return M

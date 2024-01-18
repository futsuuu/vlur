local api = vim.api

local Element = require 'shikakui.element.base'
local utils = require 'shikakui.utils'

---@class shikakui.element.Floatwin: shikakui.Element
---@field opts shikakui.element.FloatwinOpts
---@field buffer buffer
---@field window? window
local M = setmetatable({}, { __index = Element })

---@return shikakui.element.Floatwin
function M.new()
    return setmetatable({}, { __index = M })
end

---@class shikakui.element.FloatwinOpts
---@field style? 'minimal'
---@field enter? boolean
---@field focusable? boolean
---@field border? 'none'|'single'|'double'|'rounded'|'solid'|'shadow'|(string|{[1]: string, [2]: string})[]
---@field buffer? buffer
local default_opts = {
    enter = false,
    focusable = true,
}

---@param opts shikakui.element.FloatwinOpts
---@param parent shikakui.Element
---@param child shikakui.Element
function M:init(opts, parent, child)
    self.opts = vim.tbl_extend('keep', opts, default_opts)
    self.parent = parent
    self.child = child
    self.buffer = self.opts.buffer or vim.api.nvim_create_buf(false, true)
end

---@return shikakui.Range
function M:width_range()
    return utils.Range(1, math.huge)
end

---@return shikakui.Range
function M:height_range()
    return utils.Range(1, math.huge)
end

---@param area shikakui.Area
---@return shikakui.Area
function M:render(area)
    ---@type vim.api.keyset.float_config
    local config = {
        col = area.pos.x,
        row = area.pos.y,
        width = area.size.width,
        height = area.size.height,
        style = self.opts.style,
        border = self.opts.border,
        focusable = self.opts.focusable,
    }
    if area.win then
        config.relative = 'win'
        config.win = area.win
    else
        config.relative = 'editor'
    end

    local win = api.nvim_open_win(self.buffer, self.opts.enter, config)
    self.child:render {
        win = win,
        buf = self.buffer,
        pos = utils.Pos(),
        size = utils.Size(api.nvim_win_get_width(win), math.huge),
    }

    return area
end

return M

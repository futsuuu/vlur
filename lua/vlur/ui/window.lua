local api = vim.api

local vlur = require 'vlur'
local types = require 'vlur.ui.types'

local M = {}

---@class vlur.ui.Window
---@field page vlur.ui.Page
---@field buffer buffer
---@field window window?
---@field resize_required boolean
---@field redraw_required boolean
M = {}

function M:init()
    self.page = types.Page.Log
    self.buffer = api.nvim_create_buf(false, true)
    self.resize_required = false
    self.redraw_required = true

    api.nvim_create_autocmd('VimResized', {
        callback = function()
            self.resize_required = true
            self:update()
        end,
    })
    api.nvim_create_autocmd('UIEnter', {
        callback = function()
            self.redraw_required = false
        end,
    })

    return self
end

---@param page? vlur.ui.Page
function M:open(page)
    if page then
        self.page = page
    end
    self.window = api.nvim_open_win(self.buffer, true, self:config())

    return self
end

function M:hide()
    api.nvim_win_hide(self.window)
    self.window = nil

    return self
end

function M:update()
    if not self.window then
        return self
    end

    if self.resize_required then
        api.nvim_win_set_config(self.window, self:config())
    end

    if self.page == types.Page.Log then
        repeat
            local log = vlur.lib:get_log()
            if log then
                self:write_log(log)
            end
        until not log
    end

    if self.redraw_required then
        vim.cmd.redraw()
    end

    return self
end

function M:config()
    local editor_rows = vim.o.lines
    local editor_cols = vim.o.columns
    ---@type vim.api.keyset.float_config
    local cfg = {
        relative = 'editor',
        width = math.floor(editor_cols * 0.8),
        height = math.floor(editor_rows * 0.8),
        col = math.floor(editor_cols * 0.1),
        row = math.floor(editor_rows * 0.1),
    }
    self.resize_required = false
    return cfg
end

---@param log string
function M:write_log(log)
    api.nvim_buf_set_lines(
        self.buffer,
        -1,
        -1,
        false,
        vim.split(log, '\n', { trimempty = true })
    )
end

return M:init()

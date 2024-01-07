local M = {}

---@class shikakui.Size
---@field width integer
---@field height integer

---@class shikakui.Position
---@field line integer
---@field col integer

---@param buffer buffer
---@param pos shikakui.Position
---@param text string
function M.set_text(buffer, pos, text)
    local replacement = { text }
    vim.api.nvim_buf_set_text(
        buffer,
        pos.line - 1,
        pos.col - 1,
        pos.line - 1,
        pos.col - 1 + text:len(),
        replacement
    )
end

return M

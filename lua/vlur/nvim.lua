-- NOTE: For compatibility and maintainability, use vim.* as little as possible outside of this file.
local api = vim.api
local fn = vim.fn

local nvim = {}

---@type table<string, function>
nvim.plugin_loaders = {}

nvim.cache_dir = fn.stdpath 'cache' .. '/vlur'
nvim.state_dir = fn.stdpath 'state' .. '/vlur'

---@param name string
---@return any
function nvim.get_opt(name)
    return api.nvim_get_option_value(name, {})
end

---@param name string
---@param value any
function nvim.set_opt(name, value)
    api.nvim_set_option_value(name, value, {})
end

---@param command string
function nvim.exec(command)
    api.nvim_exec2(command, {})
end

---@param event string|string[]
---@param pattern string|string[]
---@param callback fun(ev: table)
---@param once boolean
---@return integer
function nvim.create_autocmd(event, pattern, callback, once)
    return api.nvim_create_autocmd(event, {
        pattern = pattern,
        callback = callback,
        once = once,
    })
end

return nvim

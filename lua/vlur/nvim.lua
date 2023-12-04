local api = vim.api
local fn = vim.fn

local nvim = {}

nvim.cache_dir = fn.stdpath 'cache' .. '/vlur'
nvim.state_dir = fn.stdpath 'state' .. '/vlur'

---@param name string
---@return any
function nvim.get_opt(name)
    return api.nvim_get_option_value(name, { scope = 'global' })
end

---@param name string
---@param value any
function nvim.set_opt(name, value)
    api.nvim_set_option_value(name, value, { scope = 'global' })
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

---@param id integer
function nvim.del_autocmd(id)
    api.nvim_del_autocmd(id)
end

---@param event string|string[]
---@return table[]
function nvim.get_autocmds(event)
    return api.nvim_get_autocmds {
        event = event,
    }
end

---@param event string|string[]
---@param group integer?
---@param data any
function nvim.exec_autocmds(event, group, data)
    api.nvim_exec_autocmds(event, {
        data = data,
        group = group,
        modeline = false,
    })
end

return nvim

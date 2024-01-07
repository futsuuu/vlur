local M = {}

---@generic T
---@param initial_state T
---@return T, fun(new_state: T)
function M.use_state(initial_state)
    local component = require('shikakui.component').rendering[1]
    if not component then
        error('use_state() was called from outside of Component')
    end
    return component:use_state(initial_state)
end

return M

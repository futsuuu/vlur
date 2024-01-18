local api = vim.api

local M = {}

---Same with `math.type`
---@param number number
---@return 'float'|'integer'
function M.number_type(number)
    return number == math.floor(number) and 'integer' or 'float'
end

---@class shikakui.Area
---@field win? window
---@field buf? buffer
---@field pos shikakui.Position
---@field size shikakui.Size

---@class shikakui.Range
---@field min integer
---@field max integer
---@overload fun(min: integer, max?: integer): shikakui.Range
M.Range = setmetatable({}, {
    __call = function(_, min, max)
        local self = {
            min = min,
            max = max or min,
        }
        return setmetatable(self, { __index = M.Range })
    end,
})

---Default value: (0, 0)
---@class shikakui.Size
---@field width integer
---@field height integer
---@overload fun(width?: integer, height?: integer): shikakui.Size
M.Size = setmetatable({}, {
    ---@param width integer
    ---@param height integer
    __call = function(_, width, height)
        local self = {
            width = width or 0,
            height = height or 0,
        }
        return setmetatable(self, { __index = M.Size })
    end,
})

---@param width integer|{ width: integer }
function M.Size:set_width(width)
    if type(width) == 'number' then
        self.width = width
    else
        self.width = width.width
    end
    return self
end

---@param width integer|{ width: integer }
function M.Size:add_width(width)
    if type(width) == 'number' then
        self:set_width(self.width + width)
    else
        self:set_width(self.width + width.width)
    end
    return self
end

---@param height integer|{ height: integer }
function M.Size:set_height(height)
    if type(height) == 'number' then
        self.height = height
    else
        self.height = height.height
    end
    return self
end

---@param height integer|{ height: integer }
function M.Size:add_height(height)
    if type(height) == 'number' then
        self:set_height(self.height + height)
    else
        self:set_height(self.height + height.height)
    end
    return self
end

---Default value: (1, 1)
---@class shikakui.Position
---@field y integer
---@field x integer
---@overload fun(x?: integer, y?: integer): shikakui.Position
M.Pos = setmetatable({}, {
    __call = function(_, x, y)
        local self = {
            x = x or 1,
            y = y or 1,
        }
        return setmetatable(self, { __index = M.Pos })
    end,
})

---@param x integer|{ x: integer }
function M.Pos:set_x(x)
    if type(x) == 'number' then
        self.x = x
    else
        self.x = x.x
    end
    return self
end

---@param x integer|{ x: integer }
function M.Pos:add_x(x)
    if type(x) == 'number' then
        self:set_x(self.x + x)
    else
        self:set_x(self.x + x.x)
    end
    return self
end

---@param y integer|{ y: integer }
function M.Pos:set_y(y)
    if type(y) == 'number' then
        self.y = y
    else
        self.y = y.y
    end
    return self
end

---@param y integer|{ y: integer }
function M.Pos:add_y(y)
    if type(y) == 'number' then
        self:set_y(self.y + y)
    else
        self:set_y(self.y + y.y)
    end
    return self
end

local function line_len(buffer, ln)
    return api.nvim_buf_get_lines(buffer, ln - 1, ln, true)[1]:len() + 1
end

---@param buffer buffer
---@param pos shikakui.Position
---@param text string
function M.set_text(buffer, pos, text)
    local replacement = { text }
    local last_line = api.nvim_buf_line_count(buffer)

    local start_pos = pos
    local end_pos = M.Pos(pos.x + text:len(), pos.y)
    if pos.y > last_line then
        start_pos = M.Pos(line_len(buffer, -1), last_line)
        end_pos = start_pos
        replacement = { (' '):rep(pos.x - 1) .. text }
        for _ = last_line, pos.y - 1 do
            table.insert(replacement, 1, '')
        end
    else
        local current_line_len = line_len(buffer, pos.y)
        if end_pos.x > current_line_len then
            end_pos.x = current_line_len
            if start_pos.x > current_line_len then
                replacement = { (' '):rep(start_pos.x - current_line_len) .. text }
                start_pos.x = current_line_len
            end
        end
    end

    api.nvim_buf_set_text(
        buffer,
        start_pos.y - 1,
        start_pos.x - 1,
        end_pos.y - 1,
        end_pos.x - 1,
        replacement
    )
end

return M

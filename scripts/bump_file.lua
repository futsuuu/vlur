local data = string.dump(loadfile(arg[1]), true)
local file = io.open(arg[2], 'wb')
file:write(data)
file:close()

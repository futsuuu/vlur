local vlur_path = '..'
vim.opt_global.runtimepath:prepend { vlur_path }
local vlur = dofile(vlur_path .. '/lua/vlur.lua')

vlur.setup({}, {})

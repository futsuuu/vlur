local g = vim.g

local vlur = require 'vlur'
local utils = require 'vlur.utils'

vim.go.packpath = vim.fn.fnamemodify(utils.file_path(), ':h') .. '/package_root'

vlur.setup()

assert(g.loaded_foo ~= nil)
assert(g.loaded_bar ~= nil)

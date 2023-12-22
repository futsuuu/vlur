local g = vim.g

local vlur = require 'vlur'

vim.go.packpath = 'tests/packpath/package_root'

vlur.setup()

assert(g.loaded_foo ~= nil)
assert(g.loaded_bar ~= nil)

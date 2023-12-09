local g = vim.g

require('vlur').setup({}, {
    default_plugins = {},
})

assert(g.loaded_man ~= nil)
assert(g.loaded_matchparen ~= nil)

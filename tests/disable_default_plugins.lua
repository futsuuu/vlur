local g = vim.g

require('vlur').setup({}, {
    default_plugins = {
        man = false,
        matchparen = false,
        netrwPlugin = true,
    },
})

assert(g.loaded_man == nil)
assert(g.loaded_matchparen == nil)
assert(g.loaded_netrwPlugin ~= nil)

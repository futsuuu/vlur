local vlur = require 'vlur'

vlur.setup {
    tokyonight = {
        path = vim.fn.stdpath 'data' .. '/tokyonight.nvim',
        install = vlur.install.git 'https://github.com/folke/tokyonight.nvim',
    },
}

vim.cmd.colorscheme 'tokyonight'
assert(vim.g.colors_name == 'tokyonight')

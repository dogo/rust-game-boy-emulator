vim.g.cargo_args = ""

vim.keymap.set("n", "<leader>rr", function()
  local args = vim.g.cargo_args or ""
  if args == "" then
    args = vim.fn.input("cargo run -- ")
    vim.g.cargo_args = args
  end
  vim.cmd("split | terminal cargo run -- " .. args)
end, { desc = "cargo run com args do projeto" })

vim.keymap.set("n", "<leader>ra", function()
  vim.g.cargo_args = vim.fn.input("cargo run -- ", vim.g.cargo_args or "")
end, { desc = "editar args do cargo run" })

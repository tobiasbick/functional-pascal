local M = {}
local scopes_view

-- Register an existing DAP client; loading this file never launches a debuggee.
function M.setup(options)
  assert(type(options) == 'table', 'FPAS debugging requires explicit setup options')
  local function absolute(value, name)
    assert(type(value) == 'string' and value ~= '', name .. ' must be an explicit path')
    return vim.fn.fnamemodify(value, ':p')
  end
  local adapter = absolute(options.adapter, 'adapter')
  local program = absolute(options.program, 'program')
  assert(program:sub(-4) == '.dll', 'program must select the compiled .dll')
  local cwd = absolute(options.cwd, 'cwd')
  local args = options.args or {}
  assert(type(args) == 'table' and vim.islist(args), 'args must be a list of strings')
  for _, argument in ipairs(args) do
    assert(type(argument) == 'string', 'args must contain only strings')
  end

  local dap = require('dap')
  dap.listeners.after.event_continued.fpas = function(session)
    if session.config.type == 'fpas' then
      session.current_frame = nil
    end
  end
  vim.filetype.add({ extension = { fpas = 'fpas' } })
  if vim.fn.expand('%:e') == 'fpas' then
    vim.bo.filetype = 'fpas'
  end
  dap.adapters.fpas = function(callback, configuration)
    if vim.fn.executable(adapter) ~= 1 then
      vim.notify('NetCoreDbg is missing or not executable: ' .. adapter, vim.log.levels.ERROR)
      return
    end
    local selected = configuration.program
    if type(selected) ~= 'string' or selected:sub(-4) ~= '.dll'
      or vim.fn.filereadable(selected) ~= 1 or vim.fn.filereadable(selected:sub(1, -5) .. '.pdb') ~= 1 then
      vim.notify('Build the selected FPAS DLL and its matching portable PDB first', vim.log.levels.ERROR)
      return
    end
    if type(configuration.cwd) ~= 'string' or vim.fn.isdirectory(configuration.cwd) ~= 1 then
      vim.notify('Select an existing FPAS working directory', vim.log.levels.ERROR)
      return
    end
    callback({
      type = 'executable',
      command = adapter,
      args = { '--interpreter=vscode' },
      options = { initialize_timeout_sec = 10, disconnect_timeout_sec = 5 },
    })
  end
  dap.configurations.fpas = { {
    type = 'fpas',
    name = 'FPAS NetCoreDbg',
    request = 'launch',
    program = program,
    cwd = cwd,
    args = vim.deepcopy(args),
    stopAtEntry = false,
    justMyCode = true,
  } }
end

function M.scopes()
  if not scopes_view then
    local dap = require('dap')
    local widgets = require('dap.ui.widgets')
    local widget = vim.tbl_extend('force', {}, widgets.scopes, {
      refresh_listener = { 'scopes', 'event_continued', 'event_exited', 'event_terminated' },
      render = function(view)
        local session = dap.session()
        if not session or session.config.type ~= 'fpas' or not session.stopped_thread_id or not session.current_frame then
          view.layer().render({ 'No stopped FPAS frame; values unavailable' })
          return
        end
        widgets.scopes.render(view)
      end,
    })
    scopes_view = widgets.sidebar(widget)
  end
  scopes_view.open()
end

return M

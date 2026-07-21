const system = {
  chdir: ['chdir(path)', 'Change the current working directory.'],
  cwd: ['cwd()', 'Return the current working directory as a Result.'],
  env_get: ['env_get(name)', 'Read an environment variable as a Result.'],
  os_arch: ['os_arch()', 'Return the operating system architecture.'],
  os_eol: ['os_eol()', 'Return the platform line ending.'],
  os_homedir: ['os_homedir()', 'Return the home directory as a Result.'],
  os_platform: ['os_platform()', 'Return the operating system platform.'],
  os_tmpdir: ['os_tmpdir()', 'Return the temporary directory.'],
  process_arch: ['process_arch()', 'Return the current process architecture.'],
  process_args: ['process_args()', 'Return the process arguments.'],
  process_kill: ['process_kill(pid[, force])', 'Terminate a process and return a Result.'],
  process_list: ['process_list()', 'List running processes.'],
  process_pid: ['process_pid()', 'Return the current process ID.'],
  process_platform: ['process_platform()', 'Return the current process platform.'],
  process_run: ['process_run(command[, args[, stdin[, timeout_ms]]])', 'Run a subprocess.'],
  sleep_ms: ['sleep_ms(ms)', 'Sleep for a number of milliseconds.'],
  time_now_ms: ['time_now_ms()', 'Return the current Unix time in milliseconds.'],
};

module.exports = { system };

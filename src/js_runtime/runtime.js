const { core } = Deno;

function argsToMessage(...args) {
  return args.map((arg) => JSON.stringify(arg)).join(" ");
}

globalThis.console = {
  log: (...args) => {
    core.print(`[out]: ${argsToMessage(...args)}\n`, false);
  },
  error: (...args) => {
    core.print(`[err]: ${argsToMessage(...args)}\n`, true);
  },
};


globalThis.io = {
   readFile: (path) => {
     return core.ops.op_read_file(path);
   },
   writeFile: (path, contents) => {
     return core.ops.op_write_file(path, contents);
   },
   removeFile: (path) => {
     return core.ops.op_remove_file(path);
  },
};

globalThis.db = {
  get: (key) => {
    return core.ops.op_db_get_value(key);
  },
  setString: (key, data) => {
    return core.ops.op_db_set_string(key, data);
  },
};

globalThis.http = {
  get: (url) => {
    return core.ops.op_http_get(url);
  },
};
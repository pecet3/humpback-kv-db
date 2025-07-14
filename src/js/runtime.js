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


globalThis.file = {
   read: (path) => {
     return core.ops.file_read(path);
   },
   write: (path, contents) => {
     return core.ops.file_write(path, contents);
   },
   remove: (path) => {
     return core.ops.file_remove(path);
  },
};

globalThis.db = {
  get: (key) => {
    return core.ops.db_get_value(key);
  },
  setString: (key, data) => {
    return core.ops.db_set_string(key, data);
  },
};

globalThis.http = {
  get: (url) => {
    return core.ops.http_get(url);
  },
};
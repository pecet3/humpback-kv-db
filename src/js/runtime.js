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
    return core.ops.op_file_read(path);
  },
  write: (path, contents) => {
    return core.ops.op_file_write(path, contents);
  },
  remove: (path) => {
    return core.ops.op_file_remove(path);
  },
};

globalThis.kv = {
  get: (key) => {
    return core.ops.op_kv_get_value(key);
  },
  set: (key, data) => {
    const type = typeof data;
    core.print(type);
    if (type === "string") {
      return core.ops.op_kv_set_string(key, data);
    }
    if (type === "number") {
      return core.ops.op_kv_set_number(key, data);
    }
  },
};

globalThis.http = {
  get: (url) => {
    return core.ops.op_http_get(url);
  },
};

globalThis._event = {
  next: () => {
    return core.ops.op_event_next();
  },
  return: (id, result) => {
    return core.ops.op_event_return(id, result);
  },
};

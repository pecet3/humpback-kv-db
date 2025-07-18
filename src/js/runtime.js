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

globalThis.sql = {
  query: (query) => {
    return core.ops.op_sql_query(query);
  },

  exec: (stmt) => {
    return core.ops.op_sql_exec(stmt);
  },
};

globalThis.kv = {
  get: (key) => {
    const kind = core.ops.op_kv_get_kind(key);
    const value = core.ops.op_kv_get_value(key);
    if (kind == "object") {
      const parsed = JSON.parse(value);
      return parsed;
    }
    return value;
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
    if (type === "object") {
      return core.ops.op_kv_set_object(key, data);
    }
  },
};

globalThis.http = {
  get: (url) => {
    return core.ops.op_http_get(url);
  },
  post: (url, body) => {
    return core.ops.op_http_post(url, body);
  },
  put: (url, body) => {
    return core.ops.op_http_put(url, body);
  },
  post: (url) => {
    return core.ops.op_http_delete(url);
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

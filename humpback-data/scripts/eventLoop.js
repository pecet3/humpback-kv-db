function handleEvent(event) {
  console.log(event.event_type);

  switch (event.event_type) {
    case "code":
      try {
        const func = new Function("", event.code);
        const result = func();
        _event.return(event.id, result);
      } catch (error) {
        console.error("Execution error:", error, `event id: ${event.id}`);
        _event.return(event.id, { error: error.message });
      }
      break;
    default:
      console.warn("Unknown event type:", event.event_type);
  }
}

while (true) {
  const event = await _event.next();
  if (event) {
    await handleEvent(event);
  }
}

// / / / / / H / A / N / D / L / 3 / R / S / / / /  / //

async function handleCode(event) {
  const func = new Function("", event.code);
  try {
    const result = await func();
    _event.return(event.id, result);
  } catch (error) {
    alert(error);
    console.error(error, `event id: ${event.id}`);
  }
  console.log(event.id, " result: ", result);
}

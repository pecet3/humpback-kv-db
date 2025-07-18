async function handleEvent(event) {
  console.log("[Event]", event.event_type);

  switch (event.event_type) {
    case "code":
      await handleCode(event);
      break;

    default:
      console.warn("⚠️ Unknown event type:", event.event_type);
      _event.return(event.id, {
        error: "Unknown event type: " + event.event_type,
      });
  }
}

async function handleCode(event) {
  const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
  const func = new AsyncFunction(event.code);

  try {
    const result = await func();
    console.log(`[Result] Event ID ${event.id}:`, result);
    _event.return(event.id, result);
  } catch (error) {
    console.error("💥 Execution error:", error, `event id: ${event.id}`);
    _event.return(event.id, { error: error.message });
  }
}

while (true) {
  const event = await _event.next();
  if (event) {
    await handleEvent(event);
  }
}

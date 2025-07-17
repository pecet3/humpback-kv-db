while (true) {
  let event = await _event.next();
  if (!event) continue;
  switch (event.kind) {
    case "code":
      handleCode(event);
  }
}

async function handleCode(event) {
  const func = new Function("", event.code);
  try {
    const result = await func();
    await _event.return(result);
  } catch (error) {
    alert(error);
    console.error(error, `event id: ${event.id}`);
  }
  console.log(event.id, " result: ", result);
}

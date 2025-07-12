console.log("Hello", "runjs!");
console.error("Boom!");


let value = db.get("test");
console.log(value);

// db.setString("test2", "hello world");
let value2 = db.get("test2");
console.log(value2);

console.log("Hello", "runjs!");
const contentt = await http.get(
  "https://deno.land/std@0.177.0/examples/welcome.ts",
);
console.log("Content from fetch", contentt);
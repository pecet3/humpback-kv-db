console.log("Hello", "runjs!");
console.error("Boom!");


let value = db.get("test");
console.log(value);

db.setString("test2", "hello world");
let value2 = db.get("test2");
console.log(value2);
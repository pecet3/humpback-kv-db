console.log("Hello", "runjs!");
console.error("Boom!");

 const path = "./log.txt";
 try {
   const contents = await file.read(path);
   console.log("Read from a file", contents);
 } catch (err) {
   console.error("Unable to read file", path, err);
 }

 await file.write(path, "I can write to a file....");
 const contents = await file.read(path);
 console.log("Read from a file", path, "contents:", contents);
 console.log("Removing file", path);

 console.log("File removed");

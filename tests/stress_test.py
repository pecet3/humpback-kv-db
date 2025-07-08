import asyncio
import random
import string
import time

HOST = '127.0.0.1'
PORT = 8080
NUM_REQUESTS = 10000

def random_string(length=16):
    return ''.join(random.choices(string.ascii_letters + string.digits, k=length))

async def send_set(index):
    reader, writer = await asyncio.open_connection(HOST, PORT)
    key = f"key{index}"
    kind = "string"
    value = f"value{index}_{random_string(32)}"
    message = f"SET {key} {kind}\n"

    writer.write(message.encode())
    await writer.drain()

    ready = await reader.readline()
    if ready.decode().strip() != "READY":
        print(f"[{index}] Unexpected response: {ready}")
        writer.close()
        await writer.wait_closed()
        return

    writer.write(value.encode())
    await writer.drain()

    response = await reader.readline()
    if response.decode().strip() != "SUCCESS":
        print(f"[{index}] Set failed: {response.decode().strip()}")

    writer.close()
    await writer.wait_closed()

async def main():
    start = time.perf_counter()
    tasks = [send_set(i) for i in range(NUM_REQUESTS)]
    await asyncio.gather(*tasks)
    duration = time.perf_counter() - start
    print(f"Sent {NUM_REQUESTS} SET requests in {duration:.2f} seconds")

if __name__ == "__main__":
    asyncio.run(main())

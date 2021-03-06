""" Code for receiving and logging the data sent from the arduino """

import asyncio
import csv
import platform
import struct
from datetime import datetime
from os import path
import subprocess

from bleak import BleakClient

SERVICE_UUID = "00002A01-0000-1000-8000-00805f9b34fb"
ADDRESS = (
    "24:62:AB:B3:79:F2"
    if platform.system() != "Darwin"
    else "008FB8DC-835E-4F23-AA5B-F316D4D69876"
)

def await_disconnect(client: BleakClient):
    loop = asyncio.get_event_loop()
    fut = loop.create_future()
    client.set_disconnected_callback(lambda client: fut.set_result([client]))
    return fut


def logger(writer: csv.DictWriter):
    def callback(sender: int, data: bytearray):
        x, y, z, roll, pitch, yaw, millis = list(struct.iter_unpack("f", data))
        now = datetime.now()
        writer.writerow([now, x[0], y[0], z[0], roll[0],
                        pitch[0], yaw[0], int(float(millis[0]))])
    return callback


CLIENT: BleakClient = None


async def run(address):
    global CLIENT
    CLIENT = BleakClient(address)
    try:
        print("conneting to device...", end="")
        await CLIENT.connect()
        print("connected!")
        await CLIENT.get_services()

        logfile = path.join(
            "logs", f"{datetime.now().strftime('%d-%m-%Y_%H%M%S')}.csv"
        )

        with open(logfile, "w") as f:
            writer = csv.writer(f, delimiter=",")
            writer.writerow(
                ["TIMESTAMP", "X", "Y", "Z", "ROLL", "PITCH", "YAW", "MILLIS"])
            print(f"writing data to {logfile}...")
            await CLIENT.start_notify(SERVICE_UUID, logger(writer))
            await await_disconnect(CLIENT)
            print("client disconnected")

        await CLIENT.stop_notify(SERVICE_UUID)

    except Exception as e:
        print(e)
    finally:
        if CLIENT.is_connected:
            await CLIENT.disconnect()
        CLIENT = None

if platform.system() == "Linux":
    subprocess.run(["bluetoothctl", "disconnect", "24:62:AB:B3:79:F2"])

try:
    while True:
        loop = asyncio.get_event_loop()
        loop.run_until_complete(run(ADDRESS))
finally:
    if CLIENT is not None and CLIENT.is_connected:
        print("closing connection...")
        loop.run_until_complete(CLIENT.disconnect())
        print("closed")

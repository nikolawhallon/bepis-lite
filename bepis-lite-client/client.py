import pyaudio
import asyncio
import sys
import websockets
import time
import os
import json
from io import BytesIO
from pydub import AudioSegment
from pydub.playback import play
import threading
import janus
import queue
import requests

_pyaudio = pyaudio.PyAudio()

TIMEOUT = 0.050

FORMAT = pyaudio.paInt16
CHANNELS = 1
RATE = 48000
CHUNK = 8000

audio_queue = asyncio.Queue()
id_queue = asyncio.Queue()

STS_URL = "wss://sts.sandbox.deepgram.com"
BEPIS_SERVER_URL = "https://wcdonaldsquest.deepgram.com"
#STS_URL = "ws://localhost:4000"
#BEPIS_SERVER_URL = "http://localhost:3000"


def callback(input_data, frame_count, time_info, status_flag):
    audio_queue.put_nowait(input_data)
    return (input_data, pyaudio.paContinue)


async def run():
    dg_api_key = os.environ.get("DEEPGRAM_API_KEY")
    if dg_api_key is None:
        print("DEEPGRAM_API_KEY env var not present")
        return

    async with websockets.connect(
        STS_URL + "/agent",
        extra_headers={"Authorization": f"Token {dg_api_key}"},
    ) as ws:

        async def microphone():
            audio = pyaudio.PyAudio()
            stream = audio.open(
                format=FORMAT,
                channels=CHANNELS,
                rate=RATE,
                input=True,
                frames_per_buffer=CHUNK,
                stream_callback=callback,
            )

            stream.start_stream()

            while stream.is_active():
                await asyncio.sleep(0.1)

            stream.stop_stream()
            stream.close()

        async def sender(ws):
            # clear the menu to give ourselves a fresh start
            response = requests.delete(BEPIS_SERVER_URL + "/menu")

            # add some items to the menu
            response = requests.post(BEPIS_SERVER_URL + "/menu/items", json = {"name": "coke", "description": "a beverage", "price": 1.50, "category": "beverage"})
            response = requests.post(BEPIS_SERVER_URL + "/menu/items", json = {"name": "pepsi", "description": "a beverage", "price": 2.50, "category": "beverage"})

            # get the full menu so that we can give it to the LLM
            response = requests.get(BEPIS_SERVER_URL + "/menu")
            menu = response.text

            # we let the bepis backend server know we have a new call,
            # and we retrieve a unique id for this/the call
            response = requests.post(BEPIS_SERVER_URL + "/calls")
            id = response.text
            id_queue.put_nowait(id)

            config_message = {
                "type": "SettingsConfiguration",
                "audio": {
                    "input": {
                        "encoding": "linear16",
                        "sample_rate": 48000,
                    },
                    "output": {
                        "encoding": "linear16",
                        "sample_rate": 16000,
                        "container": "none",
                        "buffer_size": 250,
                    },
                },
                "agent": {
                    "listen": {"model": "nova-2"},
                    "think": {
                        "provider": "anthropic",
                        "model": "claude-3-haiku-20240307",
                        "instructions": "You work taking orders at a drive-through. The menu, including the names, descriptions, and prices for the items that you sell, is as follows: " + menu,
                        # this function is what STS will call to add items to the order
                        # for this call (note the "id" portion of the path)
                        "functions": [
                            {
                                "name": "add_item",
                                "description": "Add an item to an order. Only use this function if the user has confirmed that they want to add an item to their order and that that item is on the menu.",
                                "url": BEPIS_SERVER_URL + "/calls/" + id + "/order/items",
                                "input_schema": {
                                    "type": "object",
                                    "properties": {
                                        "item": {
                                            "type": "string",
                                            "description": "The name of the item that the user would like to order. The valid values come from the names of the items on the menu.",
                                        }
                                    },
                                    "required": ["item"],
                                },
                            }
                        ],
                    },
                    "speak": {"model": "aura-asteria-en"},
                },
            }

            await ws.send(json.dumps(config_message))

            try:
                while True:
                    data = await audio_queue.get()
                    await ws.send(data)
            except Exception as e:
                print("Error while sending: ", +string(e))
                raise

        async def receiver(ws):
            id = await id_queue.get()
            try:
                speaker = Speaker()
                with speaker:
                    async for message in ws:
                        if type(message) is str:
                            print(message)

                            # check the status of the order for this call
                            response = requests.get(BEPIS_SERVER_URL + "/calls/" + id + "/order")
                            print(response.text)
                        elif type(message) is bytes:
                            await speaker.play(message)
            except Exception as e:
                print(e)

        await asyncio.wait(
            [
                asyncio.ensure_future(microphone()),
                asyncio.ensure_future(sender(ws)),
                asyncio.ensure_future(receiver(ws)),
            ]
        )


def main():
    loop = asyncio.get_event_loop()
    asyncio.get_event_loop().run_until_complete(run())


def _play(audio_out, stream, stop):
    while not stop.is_set():
        try:
            data = audio_out.sync_q.get(True, TIMEOUT)
            stream.write(data)
        except queue.Empty:
            pass


class Speaker:
    def __init__(self):
        self._queue = None
        self._stream = None
        self._thread = None
        self._stop = None

    def __enter__(self):
        self._stream = _pyaudio.open(
            format=pyaudio.paInt16,
            channels=1,
            rate=16000,
            input=False,
            output=True,
        )
        self._queue = janus.Queue()
        self._stop = threading.Event()
        self._thread = threading.Thread(
            target=_play, args=(self._queue, self._stream, self._stop), daemon=True
        )
        self._thread.start()

    def __exit__(self, exc_type, exc_value, traceback):
        self._stop.set()
        self._thread.join()
        self._stream.close()
        self._stream = None
        self._queue = None
        self._thread = None
        self._stop = None

    async def play(self, data):
        return await self._queue.async_q.put(data)


if __name__ == "__main__":
    sys.exit(main() or 0)

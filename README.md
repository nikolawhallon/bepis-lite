# bepis-lite

bepis-lite demonstrates how to use the function calling feature of the Deepgram Speech-to-Speech (STS) service in a useful, productive way.

LLM function calling offers the ability to create and execute computer-understandable tasks, such as filling out forms,
submitting orders, changing account statuses, etc. Deepgram STS offers a function calling feature where the STS service
itself will make HTTP requests to endpoints that you maintain with correctly formatted input.

bepis-lite is one example of a server and client combo, where the server hosts endpoints to submit orders, and the client
interacts with Deepgram STS. Ok, so enough abstract talk, the bepis-lite client will connect to Deepgram STS, allowing
you to have a conversation with a beverage stand bot who will sell you Coke or Pepsi. Once you've successfully requested
one of the beverages, Deepgram STS will make a call to the bepis-lite server to fullfill this order (and the bepis-lite
client will notice this as it is polling the bepis-lite server to see when this has occured).

In the real world, this could, for example, send a ticket to a restaurant kitchen for the order to be fullfilled or something.

The architecture is explained by the following diagram:

![A diagram showing the architecture of this function calling system.](./bepis-function-calling-dark.png)

## Key Parts

Let's go over some key parts of the bepis-lite client (the server is extremely simple, nothing special going on there).

First, let's note the client specifies the following urls:
```
STS_URL = "ws://localhost:5000"
BEPIS_SERVER_URL = "http://localhost:3000"
```
For a production service, one would want to change the `STS_URL` to the following: `wss://sts.sandbox.deepgram.com` and
the `BEPIS_SERVER_URL` to wherever you are hosting the bepis-lite server.

Moving along, we have two main asynchronous tasks/functions, a sender, and a receiver. The sender forwards audio from the microphone
to Deepgram STS, and the receiver receives messages from Deepgram STS - for binary messages, we forward that onwards to be
played back by the speakers, and for text messages we simply print them.

This is all in-line with the typical Deepgram STS client examples. Where this client differs is in how it handles function
calling and interactions with the bepis-lite server.

Let's start to look at the sender:
```
        async def sender(ws):
            # we let the bepis backend server know we have a new call,
            # and we retrieve a unique id for this/the call
            response = requests.post(BEPIS_SERVER_URL + "/calls")
            id = response.text
            id_queue.put_nowait(id)
```
The first thing we do is we make a `POST` request to the bepis-lite server and obtain a new call id.
We also send this id to a queue so that our other asynchronous task, the receiver, can grab it. Cool.

What is this id used for? Let's take a look at the next part of the sender function:
```
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
                        "provider": "open_ai",
                        "model": "gpt-4o",
                        "instructions": "You are a beverage seller. You only sell coke and pepsi.",
                        # this function is what STS will call to submit orders
                        # for this call (note the "id" portion of the path)
                        "functions": [
                            {
                                "name": "submit_order",
                                "description": "Submit an order for a beverage.",
                                "url": BEPIS_SERVER_URL + "/calls/" + id + "/order",
                                "parameters": {
                                    "type": "object",
                                    "properties": {
                                        "item": {
                                            "type": "string",
                                            "description": "The drink the user would like to order. The only valid values are coke or pepsi.",
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
```
Aha! So in the config message we send to Deepgram STS at the beginning of the stream, we are specifying a function,
named "submit_order", and we are specifying a URL here which specifies the endpoint `/calls/:id/order`. So Deepgram STS
can use its internal LLM function calling to call out to this endpoint, which references this specific call via the id
in the endpoint path! And how does this help us? Well let's skip ahead and look at the receiver function for a moment:
```
        async def receiver(ws):
            id = await id_queue.get()
            try:
                speaker = Speaker()
                with speaker:
                    async for message in ws:
                        if type(message) is str:
                            print(message)

                            # check if an order for this call has been submitted
                            # this url could work too: BEPIS_SERVER_URL + "/calls/" + id + "/order"
                            response = requests.get(BEPIS_SERVER_URL + "/calls/" + id)
                            print(response.text)
                        elif type(message) is bytes:
                            await speaker.play(message)
```
We see that every time we get a text message from the Deepgram STS service, we also call out to hit
the `/calls/:id` endpoint of the bepis-lite server to see if that order has been made yet. This
is some quick-and-dirty polling, and it would probably be better to spin up a dedicated task which did
this polling on a timer or something instead. But here we see how our client can inspect whether or not the function has been called (instead
of just taking the LLM's word for it - the LLM actually can often lie about this).
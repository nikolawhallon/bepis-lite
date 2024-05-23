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

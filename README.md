# eo-trader

## 1. Introduction

This document outlines the specifications for a Binary Options Trading Bot built using Rust as the programming language and Tokio Tungstenite as the WebSocket (WS) client library. The bot is designed to connect to a WebSocket server, analyze candlestick data, and execute binary options trades based on a predefined strategy.

## 2. Bot Overview

### 2.1. Bot Functionality

The bot, developed in Rust, will perform the following functions:

- Connect to a WebSocket server using the Tokio Tungstenite library.
- Analyze candlestick data from the WebSocket stream.
- Execute binary options trades based on a support and resistance strategy on the 1-minute chart.
- Verify the market trend on the 5-minute chart.

### 2.2. User Interface

The bot will not have a user interface (UI). All interactions and operations will be handled through code.

## 3. Strategy

The bot will follow a specific trading strategy:

### 3.1. Trend Verification (5-minute Chart)

- The bot will analyze the trend on the 5-minute chart to determine the market direction.
- It will only execute trades when the 5-minute chart indicates a clear trend.

### 3.2. Trade Execution (1-minute Chart)

- The bot will predict price movements on the 1-minute chart using support and resistance analysis.
- It will identify opportunities to execute "call" or "put" trades on 10-second candles based on specific conditions.

### 3.3. Trading Conditions (1-minute Chart)

The bot will invest under the following conditions on the 1-minute chart:

- If a longer tail wick is observed during an uptrend on the 5-minute chart, it will buy ("call").
- If a longer head wick is observed during a downtrend on the 5-minute chart, it will sell ("put").

## 4. WebSocket Communication

### 4.1. Initial Messages

The bot, implemented in Rust, will send the following messages in sequence upon connecting to the WebSocket server using Tokio Tungstenite:

- `{"action": "setContext", ...}`
- `{"action": "multipleAction", ...}`
- `{"action": "pong", ...}`
- `{"action": "multipleAction", ...}`
- `{"action": "historySteps", ...}`
- `{"action": "expertUnsubscribe", ...}`
- `{"action": "registerNewDeviceToken", ...}`
- `{"action": "assetHistoryCandles", ...}`

### 4.2. Candlestick Data

The WebSocket server will stream candlestick data in the following format:

```json
{"action": "candles", "message": {...}}
```

## 5. Clarification

1. **Specific Conditions for Executing "Call" or "Put" Trades on 10-Second Candles:**
   The bot will execute a "call" trade (expecting the price to go up) on the 1-minute chart when, during an uptrend on the 5-minute chart, it identifies a 10-second candlestick with a longer tail wick. Conversely, the bot will execute a "put" trade (expecting the price to go down) on the 1-minute chart when, during a downtrend on the 5-minute chart, it observes a 10-second candlestick with a longer head wick. These conditions are based on short-term candlestick patterns that align with the overall trend direction.

2. **Clear Trend on the 5-Minute Chart:**
   A "clear trend" on the 5-minute chart typically refers to a sustained movement in either an upward (bullish) or downward (bearish) direction over a reasonable timeframe. It could be determined using technical indicators, moving averages, or other trend analysis methods. A clear uptrend implies a consistent series of higher highs and higher lows, while a clear downtrend involves lower highs and lower lows. The bot should use established technical criteria to confirm such trends.

3. **Support and Resistance Strategy on the 1-Minute Chart:**
   The details of the support and resistance strategy should involve specific criteria for identifying support and resistance levels on the 1-minute chart. This might include using historical price data to locate key price levels where the asset tends to reverse or consolidate. The bot should then make trading decisions based on whether the price approaches these levels, looking for potential bounces off support or resistance.

4. **Initial Messages Sent by the Bot:**
   The initial messages sent by the bot upon connecting to the WebSocket server should include essential information such as setting the context, subscribing to specific data streams, and registering the device token. These messages ensure that the bot establishes a connection, receives relevant data, and authorizes itself with the server. The structure and content of these messages should align with the server's API documentation and authentication requirements.

5. **Candlestick Data Streamed by the WebSocket Server:**
   The format and content of the candlestick data streamed by the WebSocket server should adhere to the server's API documentation. It should include essential information such as the asset identifier, candlestick timeframe, timestamp, and price data (open, close, high, low). The candlestick data allows the bot to perform technical analysis and make trading decisions based on the latest market information.

## 6. Conclusion

This document provides a high-level overview of the Binary Options Trading Bot's specifications. Developers will use Rust as the programming language and Tokio Tungstenite as the WebSocket client library to implement the bot's functionality according to the specified strategy and WebSocket communication.

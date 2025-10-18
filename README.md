# Kafka2i - TUI for Kafka

A terminal UI for Kafka built in Rust.  
Based on 
- [ratatui](https://github.com/ratatui/ratatui): TUI Library
- [rust-rdkafka](https://github.com/fede1024/rust-rdkafka): Rust wrapper over rdkafka

![Alt Text](./kafka2i.gif)

## How to use
The simplest way to use the tool is to invoke it by providing the Kafka bootstrap URL
```
./kafka2i --bootstrap-servers <bootstrap_endpoint>
```

## Supported Commandline Args
| Argument                         | Required | Default  | Description |
|----------------------------------|----------|----------|-------------|
|--bootstrap-servers               | True     |          | Kafka boostrap server/s endpoint |
|--protocol                        | False    | SSL      | Should be one of `PLAINTEXT`, `SSL`, `SASL_SSL`, `SASL_PLAINTEXT` |
|--log-level                       | False    | info     | Should be one of `info`, `debug`, `error` |
|--group-id                        | False    | cg.krust | Consumer group id |
|--ssl-ca-location                 | False    |          | CA for server certificate validation |
|--ssl-client-key-location         | False    |          | Client private key location |
|--ssl-client-certificate-location | False    |          | Client certificate location |
|--disable-ssl-cert-vertification  | False    | false    | Disabling server cert validation |
|--sasl-mechanism                  | False    |          | Should be one of `PLAIN`, `OAUTHBEARER` |
|--sasl-username                   | False    |          | SASL username, required if sasl mechanism is `PLAIN` |
|--sasl-password                   | False    |          | SASL Password, required if sasl mechanism is `PLAIN` |
|--oauth-token-endpoint            | False    |          | Token endpoint, required if sasl mechanism is `OAUTHBEARER` |
|--oauth-client-id                 | False    |          | ClientID, required if sasl mechanism is `OAUTHBEARER` |
|--oauth-client-secret             | False    |          | ClientSecret, required if sasl mechanism is `OAUTHBEARER` |
|--oauth-scope                     | False    |          | OAuth Scope which with token is to be retrieved  |
|--https-ca-location               | False    |          | CA for server certificate validation of token endpoint |

To quick check all supported arguments, you can always run
```
./kafka2i --help
```

## Features
### Consumer Mode (default mode)
Tool starts in the consumer mode by default. Consumer is created under the consumer group provided as the argument.  
Consumer does not subscribe directly to any of the topics but assigns the required paritions when necessary

- Viewing metadata related to Brokers, Consumer Groups, Topics and Paritions
- Viewing messages for a given parition. Currently supports only messages in plain text like `JSON`
- Navigating to messages at previous or next offsets with Left/Right keys
- Seeking message at a specific offset or a timestamp
- Displayed message is by default copied to the clipboard for its usage oustide of TUI

### Producer Mode
- Not yet supported

## Key Bindings  
```
TAB        - Navigate between brokers, consumergroups, topics & paritions lists  
UP/DOWN    - Scroll through the list entries  
M          - Scroll down the message pane  
N          - Scroll up the message pane
: (colon)  - Enter edit mode  
LEFT       - Move to the previous offset of the selected parition  
RIGHT      - Move to the next offset of the selected parition  
H          - Open/Close Help Window  
```

## Commands
Commands can be entered when in edit mode. Press `:` to enter edit mode

`offset!<number>`- Provide the offset to retrieve the message from that offset. E.g. `offset!7656`  
`ts!<epoch_in_ms>`      - Provide a unix timestamp to retrieve the message. E.g. `ts!1760597487571`


## Logs
A new logfile is generated everytime the tool runs and the file is stored next to the binary
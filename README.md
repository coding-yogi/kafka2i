# Kafka2i - TUI for Kafka

A terminal UI for Kafka built in Rust &#x1f980;.  
Based on 
- [ratatui](https://github.com/ratatui/ratatui): TUI Library
- [rust-rdkafka](https://github.com/fede1024/rust-rdkafka): Rust wrapper over rdkafka

![Alt Text](./kafka2i.gif)

## How to use
The simplest way to use the tool is to invoke it by providing the Kafka bootstrap URL
```
./kafka2i --bootstrap-servers <bootstrap_endpoint>
```

To use with OAuth
```
./kafka2i --bootstrap-servers <bootstrap_endpoint> \
--protocol=SASL_SSL \
--sasl-mechanism=OAUTHBEARER \
--oauth-token-endpoint=<token_endpoint> \
--oauth-client-id=<client_id> \
--oauth-client-secret=<client_secret> \
--oauth-scope=<scopes>
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
- Supports OAuth based authentication

### Producer Mode
- WIP

### Admin Mode
- No plans to support admin mode in near future

## Key Bindings  
`Mode` defines the mode for which the key binding apply
| Key         | Mode     | Function |
|-------------|----------|----------|
|`TAB`        |both      | Navigate between the lists in `consumer` mode and the lists + inputs when in `producer` mode |
|`UP`/`DOWN`  |both      | Scroll through the list entries |
|`p`          |consumer  | Switch to `producer` mode |
|`c`          |producer  | Switch to `consumer` mode |
|`m`          |consumer  | Scroll down the message pane |
|`n`          |consumer  | Scroll up the message pane |
|`:` (colon)  |consumer  | Enter edit mode for `consumer` |
|`LEFT`       |consumer  | Move to the previous offset of the selected parition |
|`RIGHT`      |consumer  | Move to the next offset of the selected parition |
|`h`          |both      | Open/Close help window |
|`q`          |both      | Quit application (In `normal` mode) |


## Consumer Commands
Commands can be entered when in edit mode. Press `:` to enter edit mode

`offset!<number>`- Setting the offset for a selected parition to retrieve message at the set offset. E.g. `offset!7656`  
`ts!<epoch_in_ms>` - Setting the epoch timestamp in millis to retrieve the first message at the set timestamp. E.g. `ts!1760597487571`

## Logs
A new logfile is generated everytime the tool runs and the file is stored next to the binary

## Troubleshooting
To debug connectivity or other issues, enable debug logs by setting appropriate log level.
Same log-level will be set for `rdkafka`
```
./kafka2i -b <bootstrap-servers> -log-level debug
```
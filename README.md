# co2key

CLI tool to map game controller input to simulated key presses.

## Usage

First define a config file, you can use the commited json files as an example.

```
cargo run -- your_config.json
```

Optionally use one or two verbose flags to print controller events and simulated key presses.

```
cargo run -- your_config.json -v
```

## macOS

On macOS you need to enable the following setting:
Settings > Privacy and Security > Accessability > (enable for your Terminal after the prompt)

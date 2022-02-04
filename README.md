# udv-bot
Q&A bot for internal use in UDV

## Bot
### Development
1. Get yourself a token from [@Botfather](https://t.me/botfather). It looks something like `123456789:qweqweqwe`
2. Set up the environmental variable `TELOXIDE_TOKEN`
```
# Unix-like
$ export TELOXIDE_TOKEN=<Your token here>

# Windows command line
$ set TELOXIDE_TOKEN=<Your token here>

# Windows PowerShell
$ $env:TELOXIDE_TOKEN=<Your token here>
```
To start the bot run
```
cargo run -p bot
```

## CMS
### Development
To start CMS server run
```
cargo run -p cms
```

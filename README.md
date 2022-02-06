# udv-bot
Q&A bot for internal use in UDV

## Bot
### Development
_This is aimed at *NIX users, if you are using Windows ~~tough luck~~ lookup similar commands_

Install sqlite3 build dependencies and sqlite3 CLI
```
# For Debian
$ sudo apt-get install libsqlite3-dev sqlite
```
Create SQLite database and set the environmental variable `DATABASE_URL`
```
$ touch <db file path>
$ export DATABASE_URL=file:<db file path>
```
Install `diesel_cli` and run database migrations
```
$ cargo install diesel_cli --no-default-features --features sqlite
$ diesel migration run
```
Add some data to the database
```
$ sqlite <db file path>
sqlite> insert into categories(name) values("test1");
sqlite> insert into categories(name) values("test2");
sqlite> insert into questions(question, answer, category) values("funny question1", "funny an
swer1", 1);
sqlite> insert into questions(question, answer, category) values("funny question2", "funny an
swer2", 2);
```
Get yourself a token from [@Botfather](https://t.me/botfather). It looks something like `123456789:qweqweqwe`. Then set up the environmental variable `TELOXIDE_TOKEN`
```
$ export TELOXIDE_TOKEN=<Your token here>
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

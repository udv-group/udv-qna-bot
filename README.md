# udv-bot
Q&A bot for internal use at UDV

## Project structure
### Crates
* bot - telegram bot 
* cli - CLI tool for quick database backups
* cms - web server for managing bot data
* db - collections of common database related functions

### Directories
* static - directory for static [documents](https://core.telegram.org/bots/api#senddocument), that bot can answer with

## Deployment
Set the environmental variables in .env file
```
TELOXIDE_TOKEN=<Your token here>
DB_PATH=<Absoulute path to your databse>
STATIC_DIR=./static
USE_AUTH=false
```
Build images and run with docker-compose
```
$ docker-compose build
$ docker-compose run -d
```

## Development
### Bot
Install sqlite3 build dependencies and sqlite3 CLI
```
# For Debian
$ sudo apt-get install libsqlite3-dev sqlite
```
Get yourself a token from [@Botfather](https://t.me/botfather). It looks something like `123456789:qweqweqwe`

Set the environmental variables in .env file
```
TELOXIDE_TOKEN=<Your token here>
DB_PATH=<Absoulute path to your databse>
DATABASE_URL="sqlite:${DB_PATH}"
STATIC_DIR=./static
USE_AUTH=false
ROCKET_CONFIG=./cms/Rocket.toml
```
Install `sqlx-cli` and run database migrations
```
$ cargo install sqlx-cli
$ cd db
$ sqlx database setup
```
Add some data to the database
```
$ sqlite3 <db file path>
sqlite> insert into categories(name) values("test1");
sqlite> insert into categories(name) values("test2");
sqlite> insert into questions(question, answer, category) values("funny question1", "funny an
swer1", 1);
sqlite> insert into questions(question, answer, category, attachment) values("funny question2", "funny an
swer2", 2, "qwe.png");
```

To start the bot run
```
cargo run -p bot
```

### CMS
To start CMS server run
```
cargo run -p cms
```

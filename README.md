# udv-bot
Q&A bot for internal use at UDV

## Project structure
### Crates
* bot - telegram bot 
* cli - CLI tool for quick database backups
* server - web server for managing bot data
* db - collections of common database related functions

### Directories
* static - directory for static [documents](https://core.telegram.org/bots/api#senddocument), that bot can answer with

## Deployment
Set the environmental variables in .env file
```
TELOXIDE_TOKEN=<Your token here>
DB_DIR=<Absolute path to your database directory>
STATIC_DIR=./static
USE_AUTH=false
```

Create database file
```
touch <Absolute path to your database directory>/bot.db
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
DB_PATH=<Absolute path to your database>
DATABASE_URL="sqlite:<Absolute path to your database>"
STATIC_DIR=./static
USE_AUTH=false
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
To start server run
```
cargo run -p server
```
Log level can be adjusted with environmental variable RUST_LOG
```
RUST_LOG=tower_http=trace cargo run -p server
```

## Technology
### Web
- Server: [axum](https://github.com/tokio-rs/axum)
- Templates: [askama](https://github.com/djc/askama)
- Client-side rendering: [htmx](https://htmx.org)
- CSS: [Uikit](https://getuikit.com)

### Bot
- Framework: [teloxide](https://github.com/teloxide/teloxide)

### Database
- Migrations and queries: [sqlx](https://github.com/launchbadge/sqlx)
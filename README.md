# rusty-slackbot

Slack bot that can execute Rust code

## Installation

Currently this is only supported for mac/linux based systems.

### Slack Steps

1. You'll have to create an app on slack website
2. Enable **Socket Mode** and subscribe to the following bot events: `message.channels` and `app_mention`. This will generate a bot token for you that typically starts with `xoxb-`. You will need it later.
3. You should also generate an **App-Level token** with the scope `connections:write`. The generated token will start with `xapp-`.

### Repository Steps

Note: You'll have to have docker and docker-compose installed in order to run this in a container environment.

1. Clone this repository to `/opt/`
2. In the repository folder, create a file called `.env` which will contain 4 environment variables, each on its own line in the format of `key=value`:

- `RUSTY_LOG_LEVEL` which controls the logs level. Set it to DEBUG as a default.
- `PLAYGROUND_URL` which should be set to the rust playground current URL. Currently it's `https://play.rust-lang.org`
- `SLACK_BOT_TOKEN` which is the bot token you've generated before.
- `SLACK_APP_TOKEN` which is the app token you've generated before.

3. From the repository folder enter: `make install`
4. Finally start the bot with `make run`

## Bot Supported Commands

Currently the bot has the following commands supported:

- `!code`: Entering this following a new line with formatted Rust code (using 3 backticks (\`) in slack) will execute the code and will generate and `stdout` and `stderr` along with a playground link to the code.
- `!eval`: As with the previous command you should type it as `!eval` followed by new line with formatted rust code using 3 backticks. This is for code that can live inside `main()` - so you don't have type main's signature itself. It is intended for evaluating simple expressions that do not require extra functions/imports.
- `!help`:
  - `!help docs` - will output a link for rust docs
  - `!help book` - will output a link for the rust book
  - `!help <anything else>` - typing anything other than [docs, books] will display the available commands
- `@<botname>`: Will generate a nice response from the bot

## TODO

- Add support for Windows

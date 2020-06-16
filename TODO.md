
next steps:
* support multiple channels (easily done by writer.join(BotConfig::channels))

future stuff:
* Adhere to this (decent standard for bots) https://supinic.com/bot/channel-bots/levels
  * adhere to global 1 second slowmode if not VIP or mod
  * no two messages in sequence should be same (for r9k bypass)
  * add command cooldowns (per-user)
  * permissions (per-user, persistent)
  * `commands` command for printing all available commands (permission-based, e.g. a normal user won't have access to `eval`, so don't output it)
  * `help <command>` for outputting command usage info
* More complex command parsing (same style as CLI arguments)
* Bot API for exposing standard info
* Persistence (PostgreSQL? SQLite?)
* Scripting language for custom commands (LUA?)

* Maybe consider hosting the bot somewhere once it reaches this point
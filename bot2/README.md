### Setup

If you want the bot to be able to write messages into chat, you'll need a Twitch oauth token, which you can [generate here](https://twitchapps.com/tmi/).

The bot will _optionally_ read a `Config.toml` file in the root directory. You may initialize this file by copying it from `Config.example.toml` and filling in the desired values. If this file doesn't exist, default values are used, such as an anonymous login to twitch and concurrency equal to the number of logical cores available.

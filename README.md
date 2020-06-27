
![Sleep tight, aniki][aniki]

# Sleep tight, aniki


### Instructions

* clone the repo
* generate an oauth key at https://twitchapps.com/tmi/
* get your JWT token by toggling "Show Secrets" at https://streamelements.com/dashboard/account/channels
* Optionally, get a YouTube API key at https://console.developers.google.com/. See https://developers.google.com/youtube/v3/getting-started for detailed instructions. This key is required for queueing YouTube playlists.
* make a `secrets.toml` file at the root-level of the repo
* enter the following information:

    ```toml
    name = "BOT_NAME"
    oauth_token = "OAUTH_TOKEN"
    youtube_api_key = "YOUTUBE_API_KEY"  # Optional, required for the YouTube playlists feature
    stream_elements_jwt_token = "stream_elements_JWT_TOKEN"
    ```

    **Make sure that `BOT_NAME` matches the user for which the `OAUTH_TOKEN` was generated!**

* Download and install the PPGA transpiler
    ```bash
    $ cargo install --git https://github.com/OptimalStrategy/ppga/ --features=build-binary
    ```

* Transpile the commands
    ```bash
    # Linux
    $ ppga.sh --batch-output scripts/ppga

    # Window
    C:> ppga.bat --batch-output scripts/ppga
    ```

* Optionally build the documentation

    ```bash
    $ cargo doc --no-deps
    ```

    ```bash
    # Linux
    $ xdg-open target/doc/backend/index.html

    # Windows
    C:> open target/doc/backend/index.html
    ```

* Optionally enable logging by setting the RUST_LOG environment variable:

    ```bash
    # Linux
    $ export RUST_LOG=aniki,backend

    # Windows
    PS C:\\dev\\anikibot> $env:RUST_LOG="aniki"
    ```

* Run the program

    ```bash
    $ cargo run --release
    ```


## TODOs

[TODO list](./TODO.md)

[aniki]: https://i.imgur.com/LdLYvQO.png

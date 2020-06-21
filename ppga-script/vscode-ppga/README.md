# PPGa Syntax Highlighting

Installation instructions:

1. Install vsce (vscode packaging and publishing tool)

    ```bash
    $ npm install -g vsce
    ```

2. Package the extension

    ```bash
    $ vsce package
    ```

3. Install the extension

    ```bash
    $ code --install-extension ppga-0.0.1.vsix  # (or whatever version is the latest)
    ```


## Mentions
This plugin is heavily based on [VSCode Lua Plus](https://github.com/jep-a/vscode-lua-plus) by jep-a.
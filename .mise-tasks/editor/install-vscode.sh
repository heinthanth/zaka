#!/usr/bin/env bun
#MISE description="Install the VS Code extension into VS Code"
#MISE depends=["editor:build-vscode"]
#MISE dir="{{config_root}}/editors/vscode"

vsix=$(bun -p "'zaka-' + require('./package.json').version + '.vsix'")

which code || { echo "code is not on the PATH"; exit 1; }
code --install-extension $vsix --force

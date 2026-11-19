#!/usr/bin/env bun
#MISE description="Install the VS Code extension into Cursor"
#MISE depends=["editor:build-vscode"]
#MISE dir="{{config_root}}/editors/vscode"

vsix=$(bun -p "'zaka-' + require('./package.json').version + '.vsix'")

which cursor || { echo "cursor is not on the PATH"; exit 1; }
cursor --install-extension $vsix --force

#!/usr/bin/env bun
#MISE description="Install the VS Code extension"
#MISE depends=["vscode:build"]
#MISE dir="{{config_root}}/editors/vscode"

vsix=$(bun -p "'zaka-' + require('./package.json').version + '.vsix'")

if which code; then code --install-extension $vsix --force; fi
if which cursor; then cursor --install-extension $vsix --force; fi

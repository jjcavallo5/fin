#!/usr/bin/env bash

set -e

case "$(uname -s)" in
  Linux) os="unknown-linux-gnu" ;;
  Darwin) os="apple-darwin" ;;
esac

case "$(uname -m)" in
  x86_64 | amd64) arch="x86_64" ;;
  arm64 | aarch64) arch="aarch64" ;;
esac

mkdir -p "$HOME/.fin/bin"
curl -fsSL "https://github.com/jjcavallo5/fin/releases/latest/download/fin-${arch}-${os}" \
  -o "$HOME/.fin/bin/fin"
chmod +x "$HOME/.fin/bin/fin"

path_line='export PATH="$HOME/.fin/bin:$PATH"'
if ! grep -Fqx "$path_line" "$HOME/.bashrc" 2>/dev/null; then
  printf '\n%s\n' "$path_line" >> "$HOME/.bashrc"
fi

echo "fin installed to $HOME/.fin/bin/fin"

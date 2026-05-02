#!/usr/bin/env bash

inject_message() {
  local violation="$1"
  IFS='|' read -r _code _file what why how <<< "$violation"

  cat <<EOF
[O QUE]: $_file — $what
[POR QUÊ]: $why
[COMO]: $how
EOF
}

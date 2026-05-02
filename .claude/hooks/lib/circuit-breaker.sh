#!/usr/bin/env bash

STATE_DIR="/tmp/claude_hook_${PPID}"
mkdir -p "$STATE_DIR"

circuit_breaker_is_maxed() {
  local key
  key=$(echo "$1" | md5sum | cut -d' ' -f1)
  local count
  count=$(cat "$STATE_DIR/$key" 2>/dev/null || echo 0)
  [[ "$count" -ge 3 ]]
}

circuit_breaker_increment() {
  local key
  key=$(echo "$1" | md5sum | cut -d' ' -f1)
  local count
  count=$(cat "$STATE_DIR/$key" 2>/dev/null || echo 0)
  echo $((count + 1)) > "$STATE_DIR/$key"
}

circuit_breaker_reset() {
  local key
  key=$(echo "$1" | md5sum | cut -d' ' -f1)
  rm -f "$STATE_DIR/$key"
}

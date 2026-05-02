#!/usr/bin/env bash

detect_layer() {
  local path="$1"
  case "$path" in
    */src/core/*)           echo "domain"      ;;
    */src/ports/*)          echo "port"        ;;
    */src/infrastructure/*) echo "adapter"     ;;
    */src/window/*)         echo "ui"          ;;
    */src/app.rs)           echo "composition" ;;
    */tests/*)              echo "test"        ;;
    */data/resources/*)     echo "resource"    ;;
    *)                      echo "other"       ;;
  esac
}

#!/usr/bin/env bash

run_heuristics() {
  local file_path="$1"
  local layer="$2"
  local input="$3"

  local new_content
  new_content=$(echo "$input" | jq -r '.tool_input.new_string // .tool_input.content // ""')

  # 1. Driver chamado diretamente em view (apenas camada ui)
  if [[ "$layer" == "ui" ]] && echo "$new_content" | grep -qE 'self\.driver\.[a-z_]+\('; then
    echo "DRIVER_DIRECT|$file_path|self.driver chamado diretamente|language.md:threading|Envolva em spawn_driver_task (src/infrastructure/containers/background.rs)"
    return
  fi

  # 2. GTK/Adw importado na camada de domínio
  if [[ "$layer" == "domain" ]] && echo "$new_content" | grep -qE 'use (gtk4|adw|gdk4)'; then
    echo "GTK_IN_DOMAIN|$file_path|import GTK em camada de domínio|container-management.md:layer-rules|Mova lógica GTK-dependente para src/infrastructure/ ou src/window/"
    return
  fi

  # 3. String literal sem gettext em código de UI
  if [[ "$layer" == "ui" ]] \
     && echo "$new_content" | grep -qE '"[A-Z][a-zA-Z ]{3,}"' \
     && ! echo "$new_content" | grep -qE 'gettext\(|pgettext!|ngettext!'; then
    echo "MISSING_GETTEXT|$file_path|string literal sem gettext()|language.md:i18n|Use gettext(\"...\") e adicione o arquivo em po/POTFILES"
    return
  fi

  # 4. println! em código de produção
  if [[ "$layer" != "test" ]] && echo "$new_content" | grep -qE '^\s*println!'; then
    echo "PRINTLN|$file_path|println! em código de produção|observability.md|Use logger.info() ou logger.debug() via AppLogger"
    return
  fi

  # 5. .unwrap() em código de produção
  if [[ "$layer" != "test" ]] && echo "$new_content" | grep -qE '\.unwrap\(\)'; then
    echo "UNWRAP|$file_path|.unwrap() em código de produção|language.md:error-handling|Trate o Result com match ou ? operator"
    return
  fi

  # 6. tokio importado
  if echo "$new_content" | grep -qE 'use tokio|tokio::spawn|tokio::runtime'; then
    echo "TOKIO|$file_path|tokio importado|language.md:threading|tokio conflita com GLib event loop; use async_channel::bounded(1)"
    return
  fi

  # 7. #[allow(dead_code)] adicionado
  if echo "$new_content" | grep -qE '#\[allow\(dead_code\)\]'; then
    echo "DEAD_CODE|$file_path|#[allow(dead_code)] adicionado|language.md:quality|Remova o código morto ao invés de suprimir o lint"
    return
  fi

  # 8. Cor hex hardcoded em CSS
  if [[ "$file_path" == *.css ]] && echo "$new_content" | grep -qE '#[0-9a-fA-F]{3,6}'; then
    echo "HARDCODED_COLOR|$file_path|cor hex hardcoded no CSS|accessibility.md:color-tokens|Use @success_color / @warning_color / @error_color / @accent_color"
    return
  fi

  echo ""
}

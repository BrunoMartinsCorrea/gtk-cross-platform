## Summary

<!--
One or two sentences describing what this PR does and why.
Reference the issue it closes: "Closes #123"
-->

## Checklist

- [ ] `make fmt` passes (`cargo fmt --check`)
- [ ] `make lint` passes (`cargo clippy -- -D warnings`)
- [ ] `make test` passes (`cargo test`)
- [ ] All user-visible strings use `gettext!()` / `pgettext!()` / `ngettext!()`
- [ ] Blocking driver calls go through `spawn_driver_task` — no direct GTK calls from worker threads
- [ ] New interactive widgets have `set_tooltip_text` **and** `accessible::Property::Label` set
- [ ] Touch targets on new interactive elements are ≥ 44×44 sp
- [ ] Commit message(s) follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, etc.)
- [ ] `src/core/` and `src/ports/` do not import `gtk4`, `adw`, or any IO library

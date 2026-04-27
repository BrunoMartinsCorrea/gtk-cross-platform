// SPDX-License-Identifier: GPL-3.0-or-later
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

fn po_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("po")
}

fn linguas() -> Vec<String> {
    fs::read_to_string(po_dir().join("LINGUAS"))
        .expect("po/LINGUAS not found")
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

fn read_po(locale: &str) -> String {
    fs::read_to_string(po_dir().join(format!("{locale}.po")))
        .unwrap_or_else(|_| panic!("po/{locale}.po not readable"))
}

fn unescape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn extract_field(lines: &[&str], keyword: &str) -> Option<String> {
    let key_prefix = format!("{keyword} \"");
    let mut collecting = false;
    let mut result = String::new();

    for &line in lines {
        if !collecting {
            if line.starts_with(&key_prefix) && line.ends_with('"') {
                collecting = true;
                // Content starts after the keyword and opening quote, before the closing quote.
                let after_kw = &line[key_prefix.len()..line.len() - 1];
                result.push_str(&unescape(after_kw));
            }
        } else {
            let t = line.trim();
            if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
                result.push_str(&unescape(&t[1..t.len() - 1]));
            } else {
                break;
            }
        }
    }

    if collecting { Some(result) } else { None }
}

fn placeholders(s: &str) -> HashSet<String> {
    let mut set = HashSet::new();
    let mut iter = s.chars().peekable();
    while let Some(c) = iter.next() {
        if c == '{' {
            let name: String = iter.by_ref().take_while(|&ch| ch != '}').collect();
            if !name.is_empty() {
                set.insert(name);
            }
        }
    }
    set
}

fn nplurals(content: &str) -> usize {
    for line in content.lines() {
        if let Some(pos) = line.find("nplurals=") {
            let tail = &line[pos + "nplurals=".len()..];
            let digits: String = tail.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse() {
                return n;
            }
        }
    }
    panic!("nplurals declaration not found in PO header");
}

fn singular_entries(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for block in content.split("\n\n") {
        let lines: Vec<&str> = block
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect();

        if lines.iter().any(|l| l.starts_with("msgid_plural")) {
            continue; // plural block — handled separately
        }
        let Some(msgid) = extract_field(&lines, "msgid") else {
            continue;
        };
        if msgid.is_empty() {
            continue; // header entry
        }
        let msgstr = extract_field(&lines, "msgstr").unwrap_or_default();
        out.push((msgid, msgstr));
    }
    out
}

fn plural_entries(content: &str) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for block in content.split("\n\n") {
        let lines: Vec<&str> = block
            .lines()
            .filter(|l| !l.trim_start().starts_with('#'))
            .collect();

        let Some(msgid_plural) = extract_field(&lines, "msgid_plural") else {
            continue;
        };
        let msgid = extract_field(&lines, "msgid").unwrap_or_default();
        if msgid.is_empty() {
            continue; // header (shouldn't happen, but guard anyway)
        }

        let mut forms = Vec::new();
        for i in 0..10usize {
            if let Some(v) = extract_field(&lines, &format!("msgstr[{i}]")) {
                forms.push(v);
            } else {
                break;
            }
        }
        out.push((msgid_plural, forms));
    }
    out
}

#[test]
fn linguas_and_files_are_consistent() {
    let linguas = linguas();
    let listed: HashSet<_> = linguas.iter().cloned().collect();

    for locale in &linguas {
        let path = po_dir().join(format!("{locale}.po"));
        assert!(
            path.exists(),
            "po/{locale}.po is listed in LINGUAS but the file does not exist",
        );
    }

    for entry in fs::read_dir(po_dir()).expect("cannot open po/") {
        let path = entry.expect("dir entry error").path();
        if path.extension().and_then(|e| e.to_str()) != Some("po") {
            continue;
        }
        let locale = path.file_stem().unwrap().to_str().unwrap().to_owned();
        assert!(
            listed.contains(&locale),
            "po/{locale}.po exists but is not listed in LINGUAS",
        );
    }
}

#[test]
fn plural_forms_count_matches_nplurals() {
    for locale in linguas() {
        let content = read_po(&locale);
        let expected = nplurals(&content);

        for (msgid_plural, forms) in plural_entries(&content) {
            assert_eq!(
                forms.len(),
                expected,
                "[{locale}] \"{msgid_plural}\": expected {expected} plural form(s) \
                 (nplurals={expected}), found {}",
                forms.len(),
            );
        }
    }
}

#[test]
fn singular_placeholders_preserved() {
    for locale in linguas() {
        let content = read_po(&locale);

        for (msgid, msgstr) in singular_entries(&content) {
            if msgstr.is_empty() {
                continue; // untranslated — caught by a separate test
            }
            let expected = placeholders(&msgid);
            let got = placeholders(&msgstr);
            for ph in &expected {
                assert!(
                    got.contains(ph),
                    "[{locale}] msgid \"{msgid}\": \
                     placeholder {{{ph}}} is missing from msgstr \"{msgstr}\"",
                );
            }
        }
    }
}

/// Every `{placeholder}` in a plural msgid_plural must appear in at least one
/// msgstr form.  For single-form locales (nplurals=1) it must appear in ALL
/// forms, because the single form handles every value of n.
#[test]
fn plural_placeholders_preserved() {
    for locale in linguas() {
        let content = read_po(&locale);
        let n = nplurals(&content);

        for (msgid_plural, forms) in plural_entries(&content) {
            let expected = placeholders(&msgid_plural);
            if expected.is_empty() {
                continue;
            }

            for ph in &expected {
                let any = forms
                    .iter()
                    .any(|f| !f.is_empty() && placeholders(f).contains(ph));
                assert!(
                    any,
                    "[{locale}] plural \"{msgid_plural}\": \
                     placeholder {{{ph}}} is absent from every msgstr form",
                );

                // Single-form locales must carry the placeholder in msgstr[0]
                // because that one form is used for all values of n.
                if n == 1 {
                    for (i, form) in forms.iter().enumerate() {
                        if form.is_empty() {
                            continue;
                        }
                        assert!(
                            placeholders(form).contains(ph),
                            "[{locale}] plural \"{msgid_plural}\": \
                             single-form locale must have {{{ph}}} in msgstr[{i}], \
                             found \"{form}\"",
                        );
                    }
                }
            }
        }
    }
}

/// No msgstr (singular or plural) may be the empty string — that silently
/// falls back to English at runtime.
#[test]
fn no_untranslated_strings() {
    for locale in linguas() {
        let content = read_po(&locale);

        for (msgid, msgstr) in singular_entries(&content) {
            assert!(
                !msgstr.is_empty(),
                "[{locale}] msgid \"{msgid}\" has an empty msgstr (untranslated)",
            );
        }

        for (msgid_plural, forms) in plural_entries(&content) {
            for (i, form) in forms.iter().enumerate() {
                assert!(
                    !form.is_empty(),
                    "[{locale}] plural \"{msgid_plural}\": msgstr[{i}] is empty (untranslated)",
                );
            }
        }
    }
}

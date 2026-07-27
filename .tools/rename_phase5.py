#!/usr/bin/env python3
"""Phase 5 mechanical rename: xai-/grok -> weepcode, no compat shim.

Two stages over tracked files only:
  A. git mv every xai-* crate directory (and stray grok/xai-named files)
  B. ordered content replacements with sentinel protection for the real
     xAI infra domain grok.com (kept verbatim: it is a dead enterprise
     endpoint, not branding).

Idempotent: re-running rewrites nothing that is already renamed.
"""
import os
import re
import subprocess
import sys

ROOT = "/Volumes/SU710/code/WeepCode"
CRATE_PARENTS = ["crates/codegen", "crates/common", "crates/build", "prod/mc"]

# Files that must not be rewritten (history / legal / third-party records).
SKIP_PATHS = {
    "SOURCE_REV",
    "LICENSE",
    "THIRD-PARTY-NOTICES",
    "NOTICE",
    ".gitignore",
}
SKIP_DIRS = (
    "docs/process/",   # historical daily logs stay verbatim
)

TEXT_EXTS = {
    ".rs", ".toml", ".md", ".proto", ".py", ".js", ".json", ".yaml", ".yml",
    ".sh", ".txt", ".lock", ".bzl", ".ps1", ".ts", ".html", ".css",
}

RULES = [
    ("grok.com", "@@GROKDOTCOM@@"),          # sentinel: real xAI infra domain
    ("xai.grok.tools.v1", "weepcode.tools.v1"),
    ("xai-grok-", "weepcode-"),
    ("xai_grok_", "weepcode_"),
    ("xai-grok", "weepcode"),
    ("xai-", "weepcode-"),
    ("xai_", "weepcode_"),
    ("xai.grok.", "weepcode."),
    ("xai::", "weepcode::"),
    ('"xai.', '"weepcode.'),
    ("XAI_API_KEY", "WEEPCODE_API_KEY"),
    ("XAI_API_BASE_URL", "WEEPCODE_API_BASE_URL"),
    ("XAI_", "WEEPCODE_"),
    ("GROK_", "WEEPCODE_"),
    ("grok_", "weepcode_"),
    (r"\bGrok", "WeepCode"),
    ("X-XAI-", "X-WeepCode-"),
    (r"\bXAI\b", "WeepCode"),
    (r"\bxAI\b", "WeepCode"),
    ('"x.ai/', '"weepcode/'),
    (".grok/", ".weepcode/"),
    (".grok", ".weepcode"),
    (r"\bgrok\b", "weepcode"),
    ("@@GROKDOTCOM@@", "grok.com"),
]
RULES = [(re.compile(p) if p.startswith("\\b") else None, p, r) for p, r in RULES]


def new_crate_name(name: str) -> str:
    if name.startswith("xai-grok-"):
        return "weepcode-" + name[len("xai-grok-"):]
    if name.startswith("xai-"):
        return "weepcode-" + name[len("xai-"):]
    return name


def git(*args, cwd=ROOT):
    return subprocess.run(["git", *args], cwd=cwd, check=True,
                          capture_output=True, text=True).stdout


def rename_crate_dirs():
    moved = []
    for parent in CRATE_PARENTS:
        base = os.path.join(ROOT, parent)
        if not os.path.isdir(base):
            continue
        for entry in sorted(os.listdir(base)):
            if not entry.startswith("xai-"):
                continue
            new = new_crate_name(entry)
            if new == entry:
                continue
            src, dst = os.path.join(base, entry), os.path.join(base, new)
            if os.path.exists(dst):
                print(f"SKIP (exists): {dst}")
                continue
            git("mv", os.path.join(parent, entry), os.path.join(parent, new))
            moved.append((entry, new))
    # Stray grok/xai-named files outside crate dirs.
    stray = []
    for f in git("ls-files").splitlines():
        base = os.path.basename(f)
        if ("grok" in base or "xai" in base) and os.path.dirname(f) not in (
            "crates/codegen", "crates/common", "crates/build", "prod/mc"
        ):
            stray.append(f)
    for f in stray:
        d = os.path.dirname(f)
        b = os.path.basename(f)
        nb = b.replace("grok", "weepcode").replace("xai", "weepcode")
        if nb != b and not os.path.exists(os.path.join(ROOT, d, nb)):
            git("mv", f, os.path.join(d, nb))
            moved.append((f, os.path.join(d, nb)))
    return moved


def rewrite_contents():
    changed = 0
    for f in git("ls-files").splitlines():
        if f in SKIP_PATHS or f.startswith(SKIP_DIRS):
            continue
        ext = os.path.splitext(f)[1]
        if ext not in TEXT_EXTS:
            continue
        path = os.path.join(ROOT, f)
        try:
            text = open(path, encoding="utf-8").read()
        except (UnicodeDecodeError, OSError):
            continue
        orig = text
        for rx, pat, rep in RULES:
            text = rx.sub(rep, text) if rx else text.replace(pat, rep)
        if text != orig:
            open(path, "w", encoding="utf-8").write(text)
            changed += 1
    return changed


if __name__ == "__main__":
    moved = rename_crate_dirs()
    print(f"MOVED {len(moved)} paths")
    for a, b in moved:
        print(f"  {a} -> {b}")
    changed = rewrite_contents()
    print(f"REWROTE {changed} files")

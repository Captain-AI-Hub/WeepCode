#!/usr/bin/env python3
"""WeepCode G1/G2 gate smoke test (PTY end-to-end).

Spawns the freshly built TUI in a pty with a pristine HOME and verifies:
  G1: no WeepCode login wall — the provider setup form appears instead.
  G2: the form persists a [model.*] profile to ~/.weepcode/config.toml (0600),
      points [models].default at it, and a restart skips the login wall.

Usage: provider_setup_smoke.py <binary> <workdir>
Prints a transcript to <workdir>/transcript.txt and exits non-zero on failure.
"""
import os
import pty
import re
import select
import shutil
import stat
import sys
import time

TIMEOUT = 90


def set_pty_winsize(fd, rows=30, cols=100):
    """A forked pty defaults to 0x0; TUI renderers draw nothing at zero size."""
    import fcntl
    import struct
    import termios

    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))


def read_available(fd, transcript, duration):
    """Drain pty output for `duration` seconds, appending to transcript."""
    end = time.time() + duration
    while time.time() < end:
        r, _, _ = select.select([fd], [], [], 0.2)
        if fd in r:
            try:
                data = os.read(fd, 65536)
            except OSError:
                break
            if not data:
                break
            transcript.append(data)


def screen_text(transcript):
    raw = b"".join(transcript).decode("utf-8", errors="replace")
    # Strip ANSI escape sequences for content assertions.
    return re.sub(r"\x1b\[[0-9;?]*[a-zA-Z]|\x1b[()][0-9A-Z]|\x1b[>=]", "", raw)


def type_text(fd, text, delay=0.02):
    for ch in text:
        os.write(fd, ch.encode())
        time.sleep(delay)


def main():
    binary, workdir = sys.argv[1], sys.argv[2]
    home = os.path.join(workdir, "home")
    weepcode_home = os.path.join(home, ".weepcode")
    shutil.rmtree(home, ignore_errors=True)
    os.makedirs(weepcode_home)

    env = dict(
        os.environ,
        HOME=home,
        WEEPCODE_HOME=weepcode_home,
        TERM="xterm-256color",
        # No WEEPCODE_API_KEY / WEEPCODE_* credentials on purpose.
    )
    env.pop("WEEPCODE_API_KEY", None)
    env.pop("WEEPCODE_CODE_WEEPCODE_API_KEY", None)

    pid, fd = pty.fork()
    if pid == 0:
        os.execvpe(binary, [binary], env)
    set_pty_winsize(fd)

    transcript = []
    failures = []

    # ── Phase 1: the setup form must appear (no WeepCode login wall). ──────────
    read_available(fd, transcript, 20)
    text = screen_text(transcript)
    # Span boundaries and cursor-positioning escapes eat whitespace; compare
    # on a whitespace-free view for multi-word phrases.
    text_nospace = re.sub(r"\s+", "", text)
    if "ConfigureAPIProvider" not in text_nospace:
        failures.append("G1: provider setup form did not appear at startup")
    for bad in ("grok.com", "auth.x.ai", "accounts.x.ai", "Loginwith"):
        if bad.replace(" ", "") in text_nospace:
            failures.append(f"G1: WeepCode login UI leaked: {bad!r} on screen")

    # ── Phase 2: fill the form (openai-compatible format). ────────────────
    # Focus starts on Format. Move right once: openai-responses -> openai-compatible.
    os.write(fd, b"\x1b[C")  # Right arrow
    time.sleep(0.3)
    os.write(fd, b"\t")  # Tab -> Base URL (preset https://api.openai.com/v1)
    time.sleep(0.3)
    # Replace the preset URL with a local, unreachable one (persistence must
    # not depend on the endpoint being alive).
    for _ in range(40):
        os.write(fd, b"\x7f")  # Backspace
        time.sleep(0.01)
    type_text(fd, "http://127.0.0.1:9/v1")
    os.write(fd, b"\t")  # -> API key
    type_text(fd, "sk-weepcode-smoke-test")
    os.write(fd, b"\t")  # -> Model id
    type_text(fd, "smoke-model-1")
    os.write(fd, b"\t")  # -> Display name
    type_text(fd, "Smoke Provider")
    os.write(fd, b"\t")  # -> Max context (pre-filled 200000; replace it)
    for _ in range(8):
        os.write(fd, b"\x7f")  # Backspace
        time.sleep(0.01)
    type_text(fd, "128000")
    os.write(fd, b"\r")  # Enter -> submit
    read_available(fd, transcript, 15)

    # ── Assert the profile landed on disk. ────────────────────────────────
    config_path = os.path.join(weepcode_home, "config.toml")
    if not os.path.exists(config_path):
        failures.append("G2: config.toml was not created after form submit")
        config_text = ""
    else:
        config_text = open(config_path).read()
        mode = stat.S_IMODE(os.stat(config_path).st_mode)
        if mode != 0o600:
            failures.append(f"G2: config.toml mode {oct(mode)} != 0o600")
    for needle in (
        "[model.smoke-provider]",
        'model = "smoke-model-1"',
        'name = "Smoke Provider"',
        'base_url = "http://127.0.0.1:9/v1"',
        'api_key = "sk-weepcode-smoke-test"',
        'api_backend = "chat_completions"',
        "context_window = 128000",
        '[models]',
        'default = "smoke-provider"',
    ):
        if needle not in config_text:
            failures.append(f"G2: config.toml missing {needle!r}")

    # ── Restart: eager BYOK auth must skip the setup form. ────────────────
    os.write(fd, b"\x03")  # Ctrl-C
    time.sleep(1)
    try:
        os.close(fd)
    except OSError:
        pass
    try:
        os.waitpid(pid, 0)
    except OSError:
        pass

    pid2, fd2 = pty.fork()
    if pid2 == 0:
        os.execvpe(binary, [binary], env)
    set_pty_winsize(fd2)
    transcript2 = []
    read_available(fd2, transcript2, 20)
    text2 = screen_text(transcript2)
    text2_nospace = re.sub(r"\s+", "", text2)
    if "ConfigureAPIProvider" in text2_nospace and "APIkey" in text2_nospace:
        failures.append("G2: setup form re-appeared after restart despite saved profile")
    os.write(fd2, b"\x03")
    time.sleep(1)
    try:
        os.close(fd2)
        os.waitpid(pid2, 0)
    except OSError:
        pass

    with open(os.path.join(workdir, "transcript.txt"), "w") as f:
        f.write(screen_text(transcript))
        f.write("\n\n===== RESTART =====\n\n")
        f.write(text2)

    if failures:
        print("SMOKE FAILURES:")
        for f_ in failures:
            print(" -", f_)
        sys.exit(1)
    print("SMOKE OK: G1 form shown, G2 profile persisted + restart skips setup")
    sys.exit(0)


if __name__ == "__main__":
    main()

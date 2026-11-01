#!/usr/bin/env python3
"""Runs the plugin-daw-test REAPER suite with an isolated config and the test instance hidden.

Same as reaper_qa.py, except launch(): REAPER gets its own -cfgfile (the user's REAPER settings and
session are never touched) and is hidden through System Events as soon as it starts. The check that
it is really hidden is written to hidden-check.txt. Usage: hidden_reaper.py <reaper_qa.py arguments>.
"""
import importlib.util, os, subprocess, sys, time

SKILL = os.environ.get("REAPER_QA_SCRIPT") or os.path.expanduser("~/.agent/skills/plugin-daw-test/scripts/reaper_qa.py")
spec = importlib.util.spec_from_file_location("reaper_qa", SKILL)
rq = importlib.util.module_from_spec(spec)
spec.loader.exec_module(rq)


def hidden(pid):
    script = 'tell application "System Events" to get visible of (first process whose unix id is %d)' % pid
    out = subprocess.run(["osascript", "-e", script], capture_output=True, text=True).stdout.strip()
    return out == "false"


def hide(pid):
    script = 'tell application "System Events" to set visible of (first process whose unix id is %d) to false' % pid
    subprocess.run(["osascript", "-e", script], capture_output=True, text=True)


def launch_hidden(qa_dir):
    os.makedirs(qa_dir, exist_ok=True)
    for f in ("ready.txt", "resp.txt", "cmd.txt"):
        try: os.remove(os.path.join(qa_dir, f))
        except OSError: pass
    env = dict(os.environ, PLUGIN_QA_DIR=os.path.abspath(qa_dir))
    exe = os.path.join(rq.REAPER_APP, "Contents/MacOS/REAPER")
    log = open(os.path.join(qa_dir, "reaper.log"), "w")
    proc = subprocess.Popen([exe, "-newinst", "-nosplash", "-cfgfile", "/private/tmp/radarvu-reaper-cfg/reaper.ini",
                             "-new", os.path.join(rq.HERE, "reaper_bridge.lua")],
                            env=env, stdout=log, stderr=log, start_new_session=True)
    open(os.path.join(qa_dir, "reaper.pid"), "w").write(str(proc.pid))
    end = time.time() + 120
    seen_hidden = False
    while time.time() < end:
        hide(proc.pid)
        if hidden(proc.pid): seen_hidden = True
        if os.path.exists(os.path.join(qa_dir, "ready.txt")):
            hide(proc.pid)
            open(os.path.join(qa_dir, "hidden-check.txt"), "w").write(
                "REAPER PID %d, own -cfgfile (/private/tmp/radarvu-reaper-cfg, seeded from a copy of the user settings; license kept out of the repo); hidden after start: %s\n" % (proc.pid, hidden(proc.pid)))
            return proc.pid
        if proc.poll() is not None:
            raise RuntimeError("REAPER exited during start, code %s" % proc.returncode)
        time.sleep(0.4)
    raise TimeoutError("bridge not ready after 120 s (hidden seen: %s)" % seen_hidden)


rq.launch = launch_hidden
def winlist_all_tool(qa_dir):
    """A hidden app has no on-screen windows, so the stock window list can come back empty and the editor
    check is skipped. This variant lists every window of the process."""
    exe = os.path.join(qa_dir, "winlist_all")
    if not os.path.exists(exe):
        subprocess.run(["swiftc", "-O", os.path.join(os.path.dirname(os.path.abspath(__file__)), "winlist_all.swift"),
                        "-o", exe], check=True, capture_output=True)
    return exe


rq.winlist_tool = winlist_all_tool
if __name__ == "__main__":
    sys.argv[0] = "reaper_qa.py"
    rq.main()

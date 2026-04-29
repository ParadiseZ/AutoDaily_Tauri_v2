#!/usr/bin/env python
"""
Small Windows control helper for the AutoDaily window.

It can list matching windows, print live cursor coordinates, click a coordinate,
and send keys such as F5 or F12 to the foregrounded target window.
"""

from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
from dataclasses import dataclass
from pathlib import Path
import subprocess
import time


user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32

SW_RESTORE = 9
MOUSEEVENTF_LEFTDOWN = 0x0002
MOUSEEVENTF_LEFTUP = 0x0004
PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
TH32CS_SNAPPROCESS = 0x00000002
INVALID_HANDLE_VALUE = ctypes.c_void_p(-1).value
INPUT_KEYBOARD = 1
KEYEVENTF_KEYUP = 0x0002
WM_KEYDOWN = 0x0100
WM_KEYUP = 0x0101
WM_SYSKEYDOWN = 0x0104
WM_SYSKEYUP = 0x0105

VK_CODES = {
    "f5": 0x74,
    "f12": 0x7B,
    "esc": 0x1B,
    "enter": 0x0D,
    "tab": 0x09,
}


class RECT(ctypes.Structure):
    _fields_ = [
        ("left", ctypes.c_long),
        ("top", ctypes.c_long),
        ("right", ctypes.c_long),
        ("bottom", ctypes.c_long),
    ]


class PROCESSENTRY32W(ctypes.Structure):
    _fields_ = [
        ("dwSize", wintypes.DWORD),
        ("cntUsage", wintypes.DWORD),
        ("th32ProcessID", wintypes.DWORD),
        ("th32DefaultHeapID", ctypes.POINTER(ctypes.c_ulong)),
        ("th32ModuleID", wintypes.DWORD),
        ("cntThreads", wintypes.DWORD),
        ("th32ParentProcessID", wintypes.DWORD),
        ("pcPriClassBase", ctypes.c_long),
        ("dwFlags", wintypes.DWORD),
        ("szExeFile", wintypes.WCHAR * 260),
    ]


class POINT(ctypes.Structure):
    _fields_ = [("x", ctypes.c_long), ("y", ctypes.c_long)]


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [
        ("wVk", wintypes.WORD),
        ("wScan", wintypes.WORD),
        ("dwFlags", wintypes.DWORD),
        ("time", wintypes.DWORD),
        ("dwExtraInfo", ctypes.c_size_t),
    ]


class INPUT_UNION(ctypes.Union):
    _fields_ = [("ki", KEYBDINPUT)]


class INPUT(ctypes.Structure):
    _fields_ = [("type", wintypes.DWORD), ("union", INPUT_UNION)]


EnumWindowsProc = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
EnumChildProc = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
user32.SendInput.argtypes = [wintypes.UINT, ctypes.POINTER(INPUT), ctypes.c_int]
user32.SendInput.restype = wintypes.UINT


@dataclass(frozen=True)
class WindowInfo:
    hwnd: int
    title: str
    pid: int
    process_path: str

    @property
    def process_name(self) -> str:
        if not self.process_path:
            return ""
        return Path(self.process_path).stem


def enable_dpi_awareness() -> None:
    try:
        ctypes.windll.shcore.SetProcessDpiAwareness(2)
    except Exception:
        try:
            user32.SetProcessDPIAware()
        except Exception:
            pass


def get_window_text(hwnd: int) -> str:
    length = user32.GetWindowTextLengthW(hwnd)
    if length <= 0:
        return ""
    buffer = ctypes.create_unicode_buffer(length + 1)
    user32.GetWindowTextW(hwnd, buffer, length + 1)
    return buffer.value


def get_window_pid(hwnd: int) -> int:
    pid = wintypes.DWORD()
    user32.GetWindowThreadProcessId(hwnd, ctypes.byref(pid))
    return int(pid.value)


def get_process_path(pid: int) -> str:
    handle = kernel32.OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, False, pid)
    if not handle:
        return ""
    try:
        size = wintypes.DWORD(32768)
        buffer = ctypes.create_unicode_buffer(size.value)
        ok = kernel32.QueryFullProcessImageNameW(handle, 0, buffer, ctypes.byref(size))
        return buffer.value if ok else ""
    finally:
        kernel32.CloseHandle(handle)


def process_snapshot() -> dict[int, tuple[int, str]]:
    snapshot = kernel32.CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
    if snapshot == INVALID_HANDLE_VALUE:
        return {}
    processes: dict[int, tuple[int, str]] = {}
    try:
        entry = PROCESSENTRY32W()
        entry.dwSize = ctypes.sizeof(PROCESSENTRY32W)
        if not kernel32.Process32FirstW(snapshot, ctypes.byref(entry)):
            return processes
        while True:
            processes[int(entry.th32ProcessID)] = (
                int(entry.th32ParentProcessID),
                entry.szExeFile,
            )
            if not kernel32.Process32NextW(snapshot, ctypes.byref(entry)):
                break
    finally:
        kernel32.CloseHandle(snapshot)
    return processes


def is_real_window(hwnd: int) -> bool:
    return bool(user32.IsWindowVisible(hwnd)) and not bool(user32.IsIconic(hwnd))


def collect_windows(include_hidden: bool = False) -> list[WindowInfo]:
    windows: list[WindowInfo] = []

    @EnumWindowsProc
    def callback(hwnd: int, _lparam: int) -> bool:
        if include_hidden or is_real_window(hwnd):
            title = get_window_text(hwnd)
            pid = get_window_pid(hwnd)
            windows.append(WindowInfo(hwnd, title, pid, get_process_path(pid)))
        return True

    user32.EnumWindows(callback, 0)
    return windows


def process_chain_matches(pid: int, needle: str, processes: dict[int, tuple[int, str]]) -> bool:
    if not needle:
        return False
    needle = needle.casefold()
    seen: set[int] = set()
    current = pid
    while current and current not in seen:
        seen.add(current)
        parent, exe = processes.get(current, (0, ""))
        path = get_process_path(current)
        name = Path(path).stem if path else Path(exe).stem
        if needle in name.casefold() or needle in exe.casefold() or needle in path.casefold():
            return True
        current = parent
    return False


def find_windows(
    process_part: str,
    title_part: str,
    match: str,
    title_mode: str,
    include_hidden: bool = False,
) -> list[WindowInfo]:
    windows = collect_windows(include_hidden=include_hidden)
    processes = process_snapshot()
    process_needle = process_part.casefold()
    title_needle = title_part.casefold()

    def by_process(window: WindowInfo) -> bool:
        name = window.process_name.casefold()
        path = window.process_path.casefold()
        direct = bool(process_needle) and (process_needle in name or process_needle in path)
        return direct or process_chain_matches(window.pid, process_part, processes)

    def by_title(window: WindowInfo) -> bool:
        if not title_needle:
            return False
        title = window.title.casefold()
        if title_mode == "exact":
            return title == title_needle
        return title_needle in title

    if match == "process":
        return [window for window in windows if by_process(window)]
    if match == "title":
        return [window for window in windows if by_title(window)]
    process_matches = [window for window in windows if by_process(window)]
    return process_matches or [window for window in windows if by_title(window)]


def first_window(process: str, title: str, match: str, title_mode: str) -> WindowInfo:
    matches = find_windows(process, title, match, title_mode)
    if not matches:
        matches = find_windows(process, title, match, title_mode, include_hidden=True)
    if not matches:
        raise SystemExit(
            "No visible, non-minimized windows matched "
            f"process={process!r} title={title!r} title_mode={title_mode!r} match={match!r}"
        )
    return matches[0]


def window_rect(hwnd: int) -> tuple[int, int, int, int]:
    rect = RECT()
    if not user32.GetWindowRect(hwnd, ctypes.byref(rect)):
        raise RuntimeError("GetWindowRect failed")
    return rect.left, rect.top, rect.right, rect.bottom


def client_origin(hwnd: int) -> tuple[int, int]:
    point = POINT(0, 0)
    if not user32.ClientToScreen(hwnd, ctypes.byref(point)):
        raise RuntimeError("ClientToScreen failed")
    return point.x, point.y


def foreground(hwnd: int, delay: float) -> None:
    if user32.IsIconic(hwnd):
        user32.ShowWindow(hwnd, SW_RESTORE)
    user32.SetForegroundWindow(hwnd)
    deadline = time.time() + max(delay, 0.1)
    while time.time() < deadline:
        if user32.GetForegroundWindow() == hwnd:
            break
        time.sleep(0.02)
    time.sleep(delay)


def key_lparam(vk: int, key_up: bool) -> int:
    scan = user32.MapVirtualKeyW(vk, 0)
    value = 1 | (scan << 16)
    if key_up:
        value |= 1 << 30
        value |= 1 << 31
    return value


def child_windows(hwnd: int) -> list[int]:
    children: list[int] = []

    @EnumChildProc
    def callback(child_hwnd: int, _lparam: int) -> bool:
        children.append(child_hwnd)
        return True

    user32.EnumChildWindows(hwnd, callback, 0)
    return children


def send_key_input(vk: int) -> str:
    inputs = (INPUT * 2)()
    inputs[0].type = INPUT_KEYBOARD
    inputs[0].union.ki = KEYBDINPUT(vk, 0, 0, 0, 0)
    inputs[1].type = INPUT_KEYBOARD
    inputs[1].union.ki = KEYBDINPUT(vk, 0, KEYEVENTF_KEYUP, 0, 0)
    sent = user32.SendInput(2, inputs, ctypes.sizeof(INPUT))
    if sent != 2:
        raise RuntimeError(f"SendInput sent {sent} events")
    return "SendInput"


def send_key_keybd(vk: int) -> str:
    user32.keybd_event(vk, 0, 0, 0)
    time.sleep(0.03)
    user32.keybd_event(vk, 0, KEYEVENTF_KEYUP, 0)
    return "keybd_event"


def post_key(hwnd: int, vk: int, include_children: bool) -> str:
    targets = [hwnd]
    if include_children:
        targets.extend(child_windows(hwnd))
    down = key_lparam(vk, False)
    up = key_lparam(vk, True)
    for target in targets:
        user32.PostMessageW(target, WM_KEYDOWN, vk, down)
        user32.PostMessageW(target, WM_KEYUP, vk, up)
    return f"PostMessage to {len(targets)} window(s)"


def send_key_wscript(key: str) -> str:
    sendkeys = {
        "f5": "{F5}",
        "f12": "{F12}",
        "esc": "{ESC}",
        "enter": "{ENTER}",
        "tab": "{TAB}",
    }[key.casefold()]
    command = (
        "$ws = New-Object -ComObject WScript.Shell; "
        f"$ws.SendKeys('{sendkeys}')"
    )
    subprocess.run(
        ["powershell", "-NoProfile", "-Command", command],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return "WScript.SendKeys"


def send_key(hwnd: int, key: str, method: str) -> str:
    vk = VK_CODES.get(key.casefold())
    if vk is None:
        raise SystemExit(f"Unsupported key {key!r}. Supported: {', '.join(sorted(VK_CODES))}")
    if method == "input":
        return send_key_input(vk)
    if method == "keybd":
        return send_key_keybd(vk)
    if method == "post":
        return post_key(hwnd, vk, include_children=False)
    if method == "broadcast":
        return post_key(hwnd, vk, include_children=True)
    if method == "sendkeys":
        return send_key_wscript(key)
    errors: list[str] = []
    for name, action in [
        ("input", lambda: send_key_input(vk)),
        ("keybd", lambda: send_key_keybd(vk)),
        ("sendkeys", lambda: send_key_wscript(key)),
        ("broadcast", lambda: post_key(hwnd, vk, include_children=True)),
    ]:
        try:
            return action()
        except Exception as exc:
            errors.append(f"{name}: {exc}")
    raise RuntimeError("; ".join(errors))


def click_screen(x: int, y: int) -> None:
    user32.SetCursorPos(x, y)
    time.sleep(0.05)
    user32.mouse_event(MOUSEEVENTF_LEFTDOWN, 0, 0, 0, 0)
    user32.mouse_event(MOUSEEVENTF_LEFTUP, 0, 0, 0, 0)


def cursor_position() -> tuple[int, int]:
    point = POINT()
    user32.GetCursorPos(ctypes.byref(point))
    return point.x, point.y


def print_position(
    process: str,
    title: str,
    match: str,
    title_mode: str,
    once: bool,
    interval: float,
) -> None:
    window = first_window(process, title, match, title_mode)
    hwnd = window.hwnd
    print(
        f"Tracking title={window.title!r} hwnd=0x{hwnd:08X} "
        f"pid={window.pid} process={window.process_name or '?'}"
    )
    print("Press Ctrl+C to stop.")
    while True:
        sx, sy = cursor_position()
        wl, wt, _wr, _wb = window_rect(hwnd)
        cx, cy = client_origin(hwnd)
        print(
            f"screen=({sx},{sy}) "
            f"window=({sx - wl},{sy - wt}) "
            f"client=({sx - cx},{sy - cy})",
            flush=True,
        )
        if once:
            return
        time.sleep(interval)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Click or send keys to AutoDaily on Windows.")
    parser.add_argument("--process", default="auto_daily", help="Process name/path substring.")
    parser.add_argument("--title", default="AutoDaily", help="Fallback window title substring.")
    parser.add_argument(
        "--title-mode",
        choices=["exact", "contains"],
        default="exact",
        help="Use exact title matching by default to avoid matching paths or internal windows.",
    )
    parser.add_argument(
        "--match",
        choices=["auto", "process", "title"],
        default="title",
        help="Match by title, process tree first, or process tree only.",
    )
    parser.add_argument("--list", action="store_true", help="List matching windows and exit.")
    parser.add_argument(
        "--pos",
        action="store_true",
        help="Print live cursor position as screen/window/client coordinates.",
    )
    parser.add_argument("--once", action="store_true", help="Print one coordinate sample and exit.")
    parser.add_argument("--interval", type=float, default=0.5, help="Coordinate print interval.")
    parser.add_argument("--click", nargs=2, type=int, metavar=("X", "Y"), help="Click coordinate.")
    parser.add_argument(
        "--delay",
        type=float,
        default=0.3,
        help="Seconds to wait after foregrounding before sending click/key.",
    )
    parser.add_argument(
        "--key-method",
        choices=["auto", "input", "keybd", "post", "broadcast", "sendkeys"],
        default="auto",
        help="Keyboard injection method. Try broadcast or sendkeys if F12 is ignored.",
    )
    parser.add_argument(
        "--relative",
        choices=["client", "window", "screen"],
        default="client",
        help="Coordinate mode for --click.",
    )
    parser.add_argument("--key", choices=sorted(VK_CODES), help="Send a key to the target window.")
    return parser


def main() -> int:
    enable_dpi_awareness()
    args = build_parser().parse_args()

    if args.list:
        matches = find_windows(args.process, args.title, args.match, args.title_mode)
        if not matches:
            print(
                "No visible, non-minimized windows matched "
                f"process={args.process!r} title={args.title!r} "
                f"title_mode={args.title_mode!r} match={args.match!r}"
            )
            return 1
        for window in matches:
            print(
                f"0x{window.hwnd:08X} pid={window.pid} "
                f"process={window.process_name or '?'} title={window.title}"
            )
        return 0

    if args.pos:
        print_position(
            args.process,
            args.title,
            args.match,
            args.title_mode,
            args.once,
            args.interval,
        )
        return 0

    if not args.click and not args.key:
        raise SystemExit("Nothing to do. Use --list, --pos, --click X Y, or --key f5/f12.")

    window = first_window(args.process, args.title, args.match, args.title_mode)
    hwnd = window.hwnd
    foreground(hwnd, args.delay)

    if args.click:
        x, y = args.click
        if args.relative == "client":
            ox, oy = client_origin(hwnd)
            x += ox
            y += oy
        elif args.relative == "window":
            left, top, _right, _bottom = window_rect(hwnd)
            x += left
            y += top
        click_screen(x, y)
        print(f"Clicked title={window.title!r} process={window.process_name or '?'} at screen=({x},{y})")

    if args.key:
        method = send_key(hwnd, args.key, args.key_method)
        print(
            f"Sent {args.key.upper()} via {method} "
            f"to title={window.title!r} process={window.process_name or '?'}"
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())

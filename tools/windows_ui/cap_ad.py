#!/usr/bin/env python
"""
Capture the visible AutoDaily window on Windows.

This script captures only the matching window rectangle instead of the full
desktop. The target window must be visible and not minimized.
"""

from __future__ import annotations

import argparse
import ctypes
from ctypes import wintypes
from datetime import datetime
from pathlib import Path
import sys
import time
from dataclasses import dataclass

try:
    from PIL import ImageGrab
except ImportError as exc:
    raise SystemExit(
        "Pillow is required. Install it with: python -m pip install pillow"
    ) from exc


user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32

SW_RESTORE = 9
PROCESS_QUERY_LIMITED_INFORMATION = 0x1000
TH32CS_SNAPPROCESS = 0x00000002
INVALID_HANDLE_VALUE = ctypes.c_void_p(-1).value


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


EnumWindowsProc = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)


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


def collect_windows() -> list[WindowInfo]:
    windows: list[WindowInfo] = []

    @EnumWindowsProc
    def callback(hwnd: int, _lparam: int) -> bool:
        if is_real_window(hwnd):
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
) -> list[WindowInfo]:
    windows = collect_windows()
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


def window_rect(hwnd: int) -> tuple[int, int, int, int]:
    rect = RECT()
    if not user32.GetWindowRect(hwnd, ctypes.byref(rect)):
        raise RuntimeError("GetWindowRect failed")
    return rect.left, rect.top, rect.right, rect.bottom


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


def client_rect_on_screen(hwnd: int) -> tuple[int, int, int, int]:
    rect = RECT()
    if not user32.GetClientRect(hwnd, ctypes.byref(rect)):
        raise RuntimeError("GetClientRect failed")
    point = wintypes.POINT(0, 0)
    if not user32.ClientToScreen(hwnd, ctypes.byref(point)):
        raise RuntimeError("ClientToScreen failed")
    return (
        point.x,
        point.y,
        point.x + rect.right - rect.left,
        point.y + rect.bottom - rect.top,
    )


def inset_rect(rect: tuple[int, int, int, int], inset: int) -> tuple[int, int, int, int]:
    left, top, right, bottom = rect
    return left + inset, top + inset, right - inset, bottom - inset


def validate_rect(rect: tuple[int, int, int, int]) -> None:
    left, top, right, bottom = rect
    if right <= left or bottom <= top:
        raise ValueError(f"Invalid capture rectangle: {rect}")


def save_image(image, output: Path, jpeg_quality: int) -> None:
    output.parent.mkdir(parents=True, exist_ok=True)
    ext = output.suffix.casefold()
    if ext in {".jpg", ".jpeg"}:
        image = image.convert("RGB")
        image.save(output, quality=jpeg_quality, optimize=True, progressive=True)
    else:
        # PNG keeps text crisp. compress_level=9 is smaller but still lossless.
        image.save(output, optimize=True, compress_level=9)


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Capture the visible AutoDaily window, cropped to the window or client area."
    )
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
    parser.add_argument(
        "--area",
        choices=["client", "window"],
        default="client",
        help="Capture only the app client area or the whole decorated window.",
    )
    parser.add_argument(
        "--inset",
        type=int,
        default=0,
        help="Crop this many pixels from every edge after choosing the area.",
    )
    parser.add_argument(
        "--output",
        default="tools/windows_ui/captures/autodaily_{timestamp}.jpg",
        help="Output path. {timestamp} is replaced automatically.",
    )
    parser.add_argument(
        "--jpeg-quality",
        type=int,
        default=85,
        help="JPEG quality when --output ends in .jpg/.jpeg.",
    )
    parser.add_argument(
        "--list",
        action="store_true",
        help="List matching visible windows and exit.",
    )
    parser.add_argument(
        "--no-foreground",
        action="store_true",
        help="Do not bring the target window to the foreground before screen-region capture.",
    )
    parser.add_argument(
        "--delay",
        type=float,
        default=0.3,
        help="Seconds to wait after foregrounding before capture.",
    )
    return parser


def main() -> int:
    enable_dpi_awareness()
    args = build_parser().parse_args()
    matches = find_windows(args.process, args.title, args.match, args.title_mode)

    if args.list:
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

    if not matches:
        print(
            "No visible, non-minimized windows matched "
            f"process={args.process!r} title={args.title!r} "
            f"title_mode={args.title_mode!r} match={args.match!r}",
            file=sys.stderr,
        )
        return 1
    window = matches[0]
    hwnd = window.hwnd
    if not args.no_foreground:
        foreground(hwnd, args.delay)
    rect = client_rect_on_screen(hwnd) if args.area == "client" else window_rect(hwnd)
    if args.inset:
        rect = inset_rect(rect, args.inset)
    validate_rect(rect)

    image = ImageGrab.grab(bbox=rect, all_screens=True)
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    output = Path(args.output.format(timestamp=timestamp))
    save_image(image, output, args.jpeg_quality)
    width, height = image.size
    print(
        f"Captured title={window.title!r} hwnd=0x{hwnd:08X} "
        f"pid={window.pid} process={window.process_name or '?'}"
    )
    print(f"Rect screen={rect} size={width}x{height}")
    print(f"Saved {output.resolve()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

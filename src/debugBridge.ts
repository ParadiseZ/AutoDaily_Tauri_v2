import { invoke } from "@tauri-apps/api/core";

type ConsoleLevel = "log" | "info" | "warn" | "error" | "debug";

const BRIDGE_MARKER = "__AUTO_DAILY_DEBUG_BRIDGE__";
const MAX_DETAILS_LENGTH = 8000;

function isTauriRuntime() {
    const tauriWindow = window as unknown as { __TAURI_INTERNALS__?: unknown };
    return typeof window !== "undefined" && Boolean(tauriWindow.__TAURI_INTERNALS__);
}

function toText(value: unknown): string {
    if (typeof value === "string") {
        return value;
    }

    if (value instanceof Error) {
        return [value.name, value.message, value.stack].filter(Boolean).join("\n");
    }

    if (typeof value === "undefined") {
        return "undefined";
    }

    if (value === null) {
        return "null";
    }

    if (typeof value === "object") {
        try {
            const seen = new WeakSet<object>();
            return JSON.stringify(
                value,
                (_key, currentValue) => {
                    if (typeof currentValue === "bigint") {
                        return `${currentValue.toString()}n`;
                    }

                    if (currentValue instanceof Error) {
                        return {
                            name: currentValue.name,
                            message: currentValue.message,
                            stack: currentValue.stack,
                        };
                    }

                    if (typeof currentValue === "object" && currentValue !== null) {
                        if (seen.has(currentValue)) {
                            return "[Circular]";
                        }

                        seen.add(currentValue);
                    }

                    return currentValue;
                },
                2,
            );
        } catch {
            return Object.prototype.toString.call(value);
        }
    }

    return String(value);
}

function splitArgs(args: unknown[]) {
    const parts = args.map((arg) => toText(arg));
    return {
        message: parts[0] ?? "",
        details: parts.slice(1).join("\n"),
    };
}

async function sendToTerminal(level: ConsoleLevel, args: unknown[]) {
    if (!isTauriRuntime()) {
        return;
    }

    const { message, details } = splitArgs(args);

    try {
        await invoke("frontend_debug_log_cmd", {
            level,
            message: message || "[empty console message]",
            details: details ? details.slice(0, MAX_DETAILS_LENGTH) : null,
        });
    } catch {
        // Avoid recursive logging if the bridge itself fails.
    }
}

export function setupDebugBridge() {
    if (typeof window === "undefined") {
        return;
    }

    const scopedWindow = window as unknown as Record<string, unknown>;
    if (scopedWindow[BRIDGE_MARKER]) {
        return;
    }
    scopedWindow[BRIDGE_MARKER] = true;

    const levels: ConsoleLevel[] = ["log", "info", "warn", "error", "debug"];

    for (const level of levels) {
        const original = console[level].bind(console);
        console[level] = (...args: unknown[]) => {
            original(...args);
            void sendToTerminal(level, args);
        };
    }

    window.addEventListener("error", (event) => {
        const details = [
            event.filename ? `file: ${event.filename}:${event.lineno}:${event.colno}` : "",
            event.error ? toText(event.error) : "",
        ]
            .filter(Boolean)
            .join("\n");

        void sendToTerminal("error", [event.message || "Unhandled window error", details]);
    });

    window.addEventListener("unhandledrejection", (event) => {
        void sendToTerminal("error", [
            "Unhandled promise rejection",
            toText(event.reason),
        ]);
    });

    console.info("[debug-bridge] frontend console forwarding enabled");
}

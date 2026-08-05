import { useEffect } from "react";

export function useTitle(v: unknown[]) {
  const documentDefined = typeof document !== "undefined";

  useEffect(() => {
    if (!documentDefined) return;

    const title = ["NanoPulse", ...v].reverse().join(" - ");

    if (document.title !== title) {
      document.title = title;
    }

    return () => {
      document.title = "NanoPulse";
    };
  }, [documentDefined, v]);
}

export function formatDateTime(input: string | undefined | null): string {
  if (!input || input === "") {
    return "";
  }

  const pad = (n: number) => String(n).padStart(2, "0");
  const dt = new Date(input);

  const hh = pad(dt.getHours());
  const mm = pad(dt.getMinutes());
  const ss = pad(dt.getSeconds());

  const yyyy = dt.getFullYear();
  const mo = pad(dt.getMonth() + 1);
  const dd = pad(dt.getDay());

  return `${yyyy}-${mo}-${dd} ${hh}:${mm}:${ss}`;
}

export function formatTime(input: string | undefined | null): string {
  if (!input || input === "") {
    return "";
  }

  const pad = (n: number) => String(n).padStart(2, "0");
  const dt = new Date(input);

  const hh = pad(dt.getHours());
  const mm = pad(dt.getMinutes());
  const ss = pad(dt.getSeconds());
  const ms = dt.getMilliseconds();

  return `${hh}:${mm}:${ss}.${ms}`;
}

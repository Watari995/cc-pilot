import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { Session } from "../lib/types";
import { useSessionStore } from "./use-session-store";

export function useTauriEvents() {
  const { addOrUpdateSession, removeSession, setSessions } = useSessionStore();

  useEffect(() => {
    // 初回ロード: Rust から全セッションを取得
    invoke<Session[]>("get_sessions")
      .then((sessions) => {
        setSessions(sessions);
      })
      .catch((err) => {
        console.error("Failed to get sessions:", err);
      });

    // リアルタイムイベントのリスナー
    const unlistenUpdate = listen<Session>("session-update", (event) => {
      addOrUpdateSession(event.payload);
    });

    const unlistenRemoved = listen<{ id: string }>(
      "session-removed",
      (event) => {
        removeSession(event.payload.id);
      },
    );

    return () => {
      unlistenUpdate.then((fn) => fn());
      unlistenRemoved.then((fn) => fn());
    };
  }, [addOrUpdateSession, removeSession, setSessions]);
}

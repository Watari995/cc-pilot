import { create } from "zustand";
import type { Environment, Session } from "../lib/types";

const DEFAULT_FILTER_ORDER: Environment[] = [
  "terminal",
  "vscode",
  "cursor",
  "desktop",
  "web",
];

function loadFilterOrder(): Environment[] {
  try {
    const stored = localStorage.getItem("filter-order");
    if (stored) {
      const parsed = JSON.parse(stored) as Environment[];
      if (Array.isArray(parsed) && parsed.length === DEFAULT_FILTER_ORDER.length) {
        return parsed;
      }
    }
  } catch { /* ignore */ }
  return DEFAULT_FILTER_ORDER;
}

interface SessionState {
  sessions: Map<string, Session>;
  selectedSessionId: string | null;
  environmentFilter: Environment | "all";
  filterOrder: Environment[];
  addOrUpdateSession: (session: Session) => void;
  removeSession: (id: string) => void;
  selectSession: (id: string | null) => void;
  setEnvironmentFilter: (filter: Environment | "all") => void;
  setFilterOrder: (order: Environment[]) => void;
  setSessions: (sessions: Session[]) => void;
}

export const useSessionStore = create<SessionState>((set) => ({
  sessions: new Map(),
  selectedSessionId: null,
  environmentFilter: "all",
  filterOrder: loadFilterOrder(),

  addOrUpdateSession: (session) =>
    set((state) => {
      const newSessions = new Map(state.sessions);
      newSessions.set(session.id, session);
      return { sessions: newSessions };
    }),

  removeSession: (id) =>
    set((state) => {
      const newSessions = new Map(state.sessions);
      newSessions.delete(id);
      const selectedSessionId =
        state.selectedSessionId === id ? null : state.selectedSessionId;
      return { sessions: newSessions, selectedSessionId };
    }),

  selectSession: (id) => set({ selectedSessionId: id }),

  setEnvironmentFilter: (filter) => set({ environmentFilter: filter }),

  setFilterOrder: (order) => {
    localStorage.setItem("filter-order", JSON.stringify(order));
    set({ filterOrder: order });
  },

  setSessions: (sessions) =>
    set(() => {
      const map = new Map<string, Session>();
      for (const session of sessions) {
        map.set(session.id, session);
      }
      return { sessions: map };
    }),
}));

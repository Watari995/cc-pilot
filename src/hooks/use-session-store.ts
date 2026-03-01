import { create } from "zustand";
import type { Environment, Session } from "../lib/types";

interface SessionState {
  sessions: Map<string, Session>;
  selectedSessionId: string | null;
  environmentFilter: Environment | "all";
  addOrUpdateSession: (session: Session) => void;
  removeSession: (id: string) => void;
  selectSession: (id: string | null) => void;
  setEnvironmentFilter: (filter: Environment | "all") => void;
  setSessions: (sessions: Session[]) => void;
}

export const useSessionStore = create<SessionState>((set) => ({
  sessions: new Map(),
  selectedSessionId: null,
  environmentFilter: "all",

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

  setSessions: (sessions) =>
    set(() => {
      const map = new Map<string, Session>();
      for (const session of sessions) {
        map.set(session.id, session);
      }
      return { sessions: map };
    }),
}));

import { useCallback, useRef, useState } from "react";
import { WS_BASE_URL } from "@/api/client";
import type { MetricKey, SimulateRequest, SimulationResults, WsOutgoing } from "@/api/types";

export type StreamStatus = "idle" | "connecting" | "running" | "done" | "error";

interface StreamState {
  status: StreamStatus;
  completed: number;
  total: number;
  metrics: Record<MetricKey, SimulationResults> | null;
  errorMessage: string | null;
}

const initialState: StreamState = {
  status: "idle",
  completed: 0,
  total: 0,
  metrics: null,
  errorMessage: null,
};

/**
 * Drives /ws/simulate: sends one SimulateRequest, then reflects the server's
 * progress/final/error message sequence into React state. See
 * crates/api/src/routes/simulate.rs's WsOutgoing enum for the wire protocol.
 */
export function useSimulationStream() {
  const [state, setState] = useState<StreamState>(initialState);
  const socketRef = useRef<WebSocket | null>(null);

  const run = useCallback((request: SimulateRequest) => {
    socketRef.current?.close();
    setState({ ...initialState, status: "connecting" });

    const socket = new WebSocket(`${WS_BASE_URL}/ws/simulate`);
    socketRef.current = socket;

    socket.onopen = () => {
      socket.send(JSON.stringify(request));
      setState((prev) => ({ ...prev, status: "running", total: request.config.iterations }));
    };

    socket.onmessage = (event) => {
      const message = JSON.parse(event.data) as WsOutgoing;
      if (message.type === "progress") {
        setState((prev) => ({ ...prev, completed: message.completed, total: message.total }));
      } else if (message.type === "final") {
        setState((prev) => ({ ...prev, status: "done", metrics: message.metrics }));
        socket.close();
      } else {
        setState((prev) => ({ ...prev, status: "error", errorMessage: message.message }));
        socket.close();
      }
    };

    socket.onerror = () => {
      setState((prev) => ({ ...prev, status: "error", errorMessage: "WebSocket connection failed" }));
    };
  }, []);

  const reset = useCallback(() => {
    socketRef.current?.close();
    setState(initialState);
  }, []);

  return { ...state, run, reset };
}

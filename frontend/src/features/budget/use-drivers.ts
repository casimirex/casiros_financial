import { useState } from "react";

// The API has no "list all drivers" endpoint (drivers are looked up one at a
// time by name), so this session's set of known drivers is tracked here to
// drive the line-item builder's dropdown in the sibling panel.
export interface DriversState {
  drivers: Record<string, string>;
  setDriver: (name: string, value: string) => void;
}

export function useDrivers(): DriversState {
  const [drivers, setDrivers] = useState<Record<string, string>>({});
  return {
    drivers,
    setDriver: (name, value) => setDrivers((prev) => ({ ...prev, [name]: value })),
  };
}

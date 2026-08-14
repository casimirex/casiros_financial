import type { SimulationResults } from "@/api/types";
import { toNumber } from "@/lib/format";

/**
 * Reconstructs a value distribution for one metric from the aggregate
 * statistics the API actually returns (min/p5/p25/median/p75/p95/max — see
 * SimulationResults). This is a genuine, honest constraint worth being
 * explicit about: /api/v1/simulate and /ws/simulate report the *aggregate*
 * distribution, never individual scenario values, so a per-scenario 3D
 * scatter can't be literal. Instead, each axis is independently sampled via
 * piecewise-linear interpolation between the real percentile breakpoints —
 * the per-axis shape is exact, but positions are not drawn from correlated
 * per-scenario data (the API doesn't expose that), so cross-axis
 * correlation in the resulting cloud is illustrative, not measured.
 */
export function sampleFromPercentiles(stats: SimulationResults, count: number): number[] {
  const breakpoints: [number, number][] = [
    [0, toNumber(stats.min)],
    [0.05, toNumber(stats.percentile_5)],
    [0.25, toNumber(stats.percentile_25)],
    [0.5, toNumber(stats.median)],
    [0.75, toNumber(stats.percentile_75)],
    [0.95, toNumber(stats.percentile_95)],
    [1, toNumber(stats.max)],
  ];

  const samples: number[] = [];
  for (let i = 0; i < count; i++) {
    const u = Math.random();
    let lo = breakpoints[0];
    let hi = breakpoints[breakpoints.length - 1];
    for (let j = 0; j < breakpoints.length - 1; j++) {
      if (u >= breakpoints[j][0] && u <= breakpoints[j + 1][0]) {
        lo = breakpoints[j];
        hi = breakpoints[j + 1];
        break;
      }
    }
    const [uLo, vLo] = lo;
    const [uHi, vHi] = hi;
    const span = uHi - uLo;
    const t = span === 0 ? 0 : (u - uLo) / span;
    samples.push(vLo + t * (vHi - vLo));
  }
  return samples;
}

/** Min-max normalizes `values` into `[-scale, scale]`, centered on the median breakpoint. */
export function normalizeToRange(values: number[], scale: number): number[] {
  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  return values.map((v) => ((v - min) / span) * 2 * scale - scale);
}

import { useMemo, useRef } from "react";
import { Canvas, useFrame } from "@react-three/fiber";
import { OrbitControls, Points, PointMaterial, Stars, Line } from "@react-three/drei";
import * as THREE from "three";

export interface ScenarioPoint {
  x: number;
  y: number;
  z: number;
  /** 0 (unfavorable) .. 1 (favorable), drives the point's color. */
  favorability: number;
}

const AXIS_EXTENT = 6;

// Deliberately NOT using drei's <Text> (troika-three-text): its async SDF
// font/glyph loading throws during the first commit in at least one real
// environment (headless Chromium + SwiftShader software WebGL), which
// silently unmounts the *entire* R3F scene with no console error — found by
// bisecting this component down to a single hardcoded mesh and adding
// pieces back one at a time. An HTML overlay (below, outside the Canvas)
// sidesteps SDF font loading entirely and renders crisper text besides.
function AxisLine({ direction }: { direction: [number, number, number] }) {
  const end: [number, number, number] = [
    direction[0] * AXIS_EXTENT,
    direction[1] * AXIS_EXTENT,
    direction[2] * AXIS_EXTENT,
  ];
  return <Line points={[[0, 0, 0], end]} color="#3d4a78" lineWidth={1} transparent opacity={0.5} />;
}

function Field({ points }: { points: ScenarioPoint[] }) {
  const groupRef = useRef<THREE.Group>(null);

  const { positions, colors } = useMemo(() => {
    const positions = new Float32Array(points.length * 3);
    const colors = new Float32Array(points.length * 3);
    const favorable = new THREE.Color("#34d399");
    const unfavorable = new THREE.Color("#fb7185");
    const tmp = new THREE.Color();

    points.forEach((p, i) => {
      positions[i * 3] = p.x;
      positions[i * 3 + 1] = p.y;
      positions[i * 3 + 2] = p.z;
      tmp.copy(unfavorable).lerp(favorable, p.favorability);
      colors[i * 3] = tmp.r;
      colors[i * 3 + 1] = tmp.g;
      colors[i * 3 + 2] = tmp.b;
    });
    return { positions, colors };
  }, [points]);

  useFrame((_, delta) => {
    if (groupRef.current) {
      groupRef.current.rotation.y += delta * 0.06;
    }
  });

  return (
    <group ref={groupRef}>
      <Points positions={positions} colors={colors} stride={3}>
        <PointMaterial
          transparent
          vertexColors
          size={0.065}
          sizeAttenuation
          depthWrite={false}
          opacity={0.85}
        />
      </Points>
    </group>
  );
}

function AxisCaptions({ axisLabels }: { axisLabels: { x: string; y: string; z: string } }) {
  return (
    <div className="pointer-events-none absolute inset-x-0 bottom-3 flex justify-center gap-4 font-mono text-[11px] uppercase tracking-wider">
      <span className="rounded-full border border-signal-500/30 bg-void-950/70 px-2.5 py-1 text-signal-400">
        X · {axisLabels.x}
      </span>
      <span className="rounded-full border border-nova-500/30 bg-void-950/70 px-2.5 py-1 text-nova-400">
        Y · {axisLabels.y}
      </span>
      <span className="rounded-full border border-caution-500/30 bg-void-950/70 px-2.5 py-1 text-caution-500">
        Z · {axisLabels.z}
      </span>
    </div>
  );
}

export function ScenarioField({
  points,
  axisLabels,
}: {
  points: ScenarioPoint[];
  axisLabels: { x: string; y: string; z: string };
}) {
  return (
    <div className="relative h-full w-full">
      <Canvas camera={{ position: [9, 6, 9], fov: 50 }} dpr={[1, 2]}>
        <color attach="background" args={["#05060c"]} />
        <fog attach="fog" args={["#05060c", 14, 26]} />
        <ambientLight intensity={0.4} />
        <Stars radius={60} depth={40} count={2000} factor={2} saturation={0} fade speed={0.4} />

        <AxisLine direction={[1, 0, 0]} />
        <AxisLine direction={[-1, 0, 0]} />
        <AxisLine direction={[0, 1, 0]} />
        <AxisLine direction={[0, -1, 0]} />
        <AxisLine direction={[0, 0, 1]} />
        <AxisLine direction={[0, 0, -1]} />

        <Field points={points} />

        <OrbitControls
          enableDamping
          dampingFactor={0.08}
          rotateSpeed={0.5}
          minDistance={4}
          maxDistance={30}
        />
      </Canvas>
      <AxisCaptions axisLabels={axisLabels} />
    </div>
  );
}

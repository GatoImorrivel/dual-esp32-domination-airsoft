export type TimeField = "red" | "blue";

/**
 * When linked, editing one field returns both values equal to the new value.
 * When enabling link, both sync to red (master).
 */
export function syncTeamTimes(
  linked: boolean,
  source: TimeField,
  redSecs: number,
  blueSecs: number,
  newValue?: number
): { redSecs: number; blueSecs: number } {
  if (linked) {
    const value =
      newValue ?? (source === "red" ? redSecs : blueSecs);
    const clamped = Math.max(1, value);
    return { redSecs: clamped, blueSecs: clamped };
  }

  if (newValue === undefined) {
    return { redSecs, blueSecs };
  }

  const clamped = Math.max(1, newValue);
  if (source === "red") {
    return { redSecs: clamped, blueSecs };
  }
  return { redSecs, blueSecs: clamped };
}

export function onLinkEnabled(redSecs: number, blueSecs: number): {
  redSecs: number;
  blueSecs: number;
} {
  const master = Math.max(1, redSecs);
  return { redSecs: master, blueSecs: master };
}

import { describe, expect, it } from "vitest";
import { onLinkEnabled, syncTeamTimes } from "./linkedTimes";

describe("linkedTimes", () => {
  it("syncs both when linked", () => {
    expect(syncTeamTimes(true, "red", 10, 20, 30)).toEqual({
      redSecs: 30,
      blueSecs: 30,
    });
  });

  it("updates only red when unlinked", () => {
    expect(syncTeamTimes(false, "red", 10, 20, 15)).toEqual({
      redSecs: 15,
      blueSecs: 20,
    });
  });

  it("updates only blue when unlinked", () => {
    expect(syncTeamTimes(false, "blue", 10, 20, 25)).toEqual({
      redSecs: 10,
      blueSecs: 25,
    });
  });

  it("clamps minimum to 1", () => {
    expect(syncTeamTimes(true, "red", 10, 10, 0)).toEqual({
      redSecs: 1,
      blueSecs: 1,
    });
  });

  it("onLinkEnabled uses red as master", () => {
    expect(onLinkEnabled(12, 99)).toEqual({ redSecs: 12, blueSecs: 12 });
  });
});

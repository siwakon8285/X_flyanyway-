import { formatCount } from "@/lib/motion/formatCount";

describe("formatCount", () => {
  it("rounds to the requested precision and preserves affixes", () => {
    expect(
      formatCount(91.956, { decimals: 1, prefix: "~", suffix: "%" }),
    ).toBe("~92.0%");
  });

  it("uses integer output by default", () => {
    expect(formatCount(155.6)).toBe("156");
  });
});

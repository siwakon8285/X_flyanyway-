import fs from "node:fs";
import path from "node:path";

describe("API client mutation navigation", () => {
  it("does not use a hard browser reload to synchronize detail state", () => {
    const source = fs.readFileSync(
      path.join(process.cwd(), "src/components/admin/api-clients/ApiClientEditor.tsx"),
      "utf8",
    );

    expect(source).not.toMatch(/(?:window\.)?location\.reload\s*\(/);
  });
});

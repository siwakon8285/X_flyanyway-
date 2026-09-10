import { readFileSync } from "node:fs";
import { join } from "node:path";

const channel = (value: number) => {
  const normalized = value / 255;
  return normalized <= 0.04045 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4;
};

const luminance = (color: string) => {
  const [red, green, blue] = color.match(/\d+/g)?.slice(0, 3).map(Number) ?? [];
  if ([red, green, blue].some((value) => value === undefined)) throw new Error(`Unsupported color: ${color}`);
  return 0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue);
};

const contrast = (foreground: string, background: string) => {
  const lighter = Math.max(luminance(foreground), luminance(background));
  const darker = Math.min(luminance(foreground), luminance(background));
  return (lighter + 0.05) / (darker + 0.05);
};

describe("API client lifecycle action styles", () => {
  beforeAll(() => {
    const style = document.createElement("style");
    style.textContent = readFileSync(
      join(process.cwd(), "src/components/admin/api-clients/apiClientOperations.css"),
      "utf8",
    );
    document.head.append(style);
  });

  it("renders the enabled revoke trigger with readable contrast and a visible destructive border", () => {
    document.body.innerHTML = '<section class="xac-terminal"><div class="xac-editor-actions"><button class="is-danger">Revoke client</button></div></section>';
    const button = document.querySelector("button") as HTMLButtonElement;
    const style = getComputedStyle(button);
    expect(style.backgroundColor).toBe("rgb(255, 253, 247)");
    expect(style.color).toBe("rgb(127, 29, 29)");
    expect(contrast(style.color, style.backgroundColor)).toBeGreaterThanOrEqual(4.5);
    expect(style.borderTopColor).not.toBe("transparent");
    expect(style.borderTopStyle).not.toBe("none");
  });
});

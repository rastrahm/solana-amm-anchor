import { describe, expect, it } from "vitest";
import { dictionaries } from "@/lib/i18n/dictionaries";

describe("i18n dictionaries", () => {
  it("exposes help content in both locales", () => {
    expect(dictionaries.es.helpTitle).toMatch(/cómo usar|demo/i);
    expect(dictionaries.en.helpTitle).toMatch(/how to use|demo/i);
    expect(dictionaries.es.helpOpen).toMatch(/ayuda/i);
    expect(dictionaries.en.helpOpen).toMatch(/help/i);
    expect(dictionaries.es.help.steps.length).toBeGreaterThan(3);
    expect(dictionaries.en.help.steps.length).toBe(
      dictionaries.es.help.steps.length
    );
    expect(dictionaries.es.help.phantom.length).toBe(
      dictionaries.en.help.phantom.length
    );
  });
});

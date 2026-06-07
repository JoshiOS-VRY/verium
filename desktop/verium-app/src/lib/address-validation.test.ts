import { describe, expect, it } from "vitest";
import { validateSendAddress } from "@/lib/address-validation";

const VALID = "VRq98Nm2P6anLHPgnHdb6NnibJ6GoG3Jm9";

describe("validateSendAddress", () => {
  it("accepts a valid 34-character V address", () => {
    expect(validateSendAddress(VALID)).toBeNull();
  });

  it("rejects missing V prefix", () => {
    expect(validateSendAddress(VALID.slice(1))).toMatch(/start with V/);
  });

  it("rejects wrong length", () => {
    expect(validateSendAddress("VY6E3KSqrMk1hcy5Cu4EGyHrdDS5ch3YH")).toMatch(
      /exactly 34/,
    );
  });

  it("rejects special characters", () => {
    expect(validateSendAddress(`${VALID.slice(0, 33)}!`)).toMatch(
      /letters and numbers/,
    );
  });
});

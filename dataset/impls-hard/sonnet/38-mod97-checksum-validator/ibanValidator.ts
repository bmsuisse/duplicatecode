/** Remove spaces and hyphens and upper-case the input. */
export function normalizeIban(input: string): string {
  return input.replace(/[\s-]+/g, "").toUpperCase();
}

/** Check an IBAN using its shape and the ISO 7064 mod-97 checksum. */
export function isValidIban(input: string): boolean {
  const iban = normalizeIban(input);
  if (!/^[A-Z]{2}\d{2}[A-Z0-9]{11,30}$/.test(iban)) return false;
  const rearranged = iban.slice(4) + iban.slice(0, 4);
  let remainder = 0;
  for (const char of rearranged) {
    // Letters map to 10..35, so they contribute two digits.
    const digits = String(Number.parseInt(char, 36));
    for (const digit of digits) {
      remainder = (remainder * 10 + Number(digit)) % 97;
    }
  }
  return remainder === 1;
}

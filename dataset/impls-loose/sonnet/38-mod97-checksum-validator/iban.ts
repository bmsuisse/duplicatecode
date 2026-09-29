/** Validate an IBAN using the ISO 7064 mod-97-10 checksum. */
export function isValidIban(input: string): boolean {
  const iban = input.replace(/[\s-]/g, "").toUpperCase();
  if (!/^[A-Z]{2}\d{2}[A-Z0-9]{1,30}$/.test(iban)) return false;

  const rearranged = iban.slice(4) + iban.slice(0, 4);
  // Compute the remainder piecewise so we never exceed Number precision.
  let remainder = 0;
  for (const ch of rearranged) {
    const digits = String(Number.parseInt(ch, 36));
    for (const d of digits) remainder = (remainder * 10 + Number(d)) % 97;
  }
  return remainder === 1;
}

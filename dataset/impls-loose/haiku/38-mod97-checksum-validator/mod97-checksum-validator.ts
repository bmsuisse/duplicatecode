export function validateMod97(iban: string): boolean {
  const cleaned = iban.replace(/[\s-]/g, "").toUpperCase();

  // Move first 4 chars to end
  const rearranged = cleaned.slice(4) + cleaned.slice(0, 4);

  // Replace letters with numbers
  let numeric = "";
  for (const char of rearranged) {
    if (/\d/.test(char)) {
      numeric += char;
    } else {
      numeric += (char.charCodeAt(0) - "A".charCodeAt(0) + 10).toString();
    }
  }

  // Calculate mod 97
  return BigInt(numeric) % 97n === 1n;
}

export function validateMod97(accountNumber: string): boolean {
  const clean = accountNumber.replace(/[\s-]/g, "").toUpperCase();

  if (clean.length < 15 || clean.length > 34) {
    return false;
  }

  // Move first 4 characters to end
  const rearranged = clean.slice(4) + clean.slice(0, 4);

  // Convert letters to numbers
  let numeric = "";
  for (const char of rearranged) {
    if (/\d/.test(char)) {
      numeric += char;
    } else if (/[A-Z]/.test(char)) {
      numeric += (char.charCodeAt(0) - 65 + 10).toString();
    } else {
      return false;
    }
  }

  // Check mod 97
  return BigInt(numeric) % BigInt(97) === BigInt(1);
}

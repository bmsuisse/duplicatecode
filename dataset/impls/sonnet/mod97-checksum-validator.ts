const IBAN_PATTERN = /^[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}$/;

function normalize(value: string): string {
  return value.replace(/[\s-]/g, "").toUpperCase();
}

export function isValidIban(value: string): boolean {
  const candidate = normalize(value);
  if (!IBAN_PATTERN.test(candidate)) {
    return false;
  }
  const rearranged = candidate.slice(4) + candidate.slice(0, 4);
  let remainder = 0;
  for (const char of rearranged) {
    const digits = String(Number.parseInt(char, 36));
    for (const digit of digits) {
      remainder = (remainder * 10 + Number(digit)) % 97;
    }
  }
  return remainder === 1;
}

export function formatIban(value: string): string {
  if (!isValidIban(value)) {
    throw new Error(`Invalid IBAN: ${value}`);
  }
  return (normalize(value).match(/.{1,4}/g) ?? []).join(" ");
}

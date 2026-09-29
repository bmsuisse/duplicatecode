export function isValidIban(value: string): boolean {
  const iban = value.replace(/[\s-]/g, "").toUpperCase();

  if (!/^[A-Z]{2}[0-9]{2}[A-Z0-9]{11,30}$/.test(iban)) {
    return false;
  }

  // Move first 4 characters to the end
  const moved = iban.slice(4) + iban.slice(0, 4);

  // Replace letters with numbers
  let numericString = "";
  for (const char of moved) {
    if (char >= "0" && char <= "9") {
      numericString += char;
    } else {
      numericString += (char.charCodeAt(0) - "A".charCodeAt(0) + 10).toString();
    }
  }

  // Compute mod 97 piecewise
  let remainder = 0;
  for (const digit of numericString) {
    remainder = (remainder * 10 + parseInt(digit, 10)) % 97;
  }

  return remainder === 1;
}

export function formatIban(value: string): string {
  if (!isValidIban(value)) {
    throw new Error("Invalid IBAN");
  }

  const iban = value.replace(/[\s-]/g, "").toUpperCase();
  const groups: string[] = [];
  for (let i = 0; i < iban.length; i += 4) {
    groups.push(iban.slice(i, i + 4));
  }
  return groups.join(" ");
}

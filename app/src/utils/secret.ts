// Même alphabet que le secret créé par Baskul (core/src/config.rs) : sans caractères ambigus
const SECRET_ALPHABET = "abcdefghjkmnpqrstuvwxyz23456789";

export function newSecret() {
  const bytes = crypto.getRandomValues(new Uint8Array(24));
  const chars = Array.from(bytes, byte => SECRET_ALPHABET[byte % SECRET_ALPHABET.length]);
  return Array.from({ length: 6 }, (_, i) => chars.slice(i * 4, i * 4 + 4).join("")).join("-");
}

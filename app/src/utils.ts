/** Copie un texte, avec un repli si l'API du presse-papiers est refusée. */
export async function copyText(text: string): Promise<void> {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const area = document.createElement("textarea");
    area.value = text;
    area.style.position = "fixed";
    area.style.opacity = "0";
    document.body.append(area);
    area.select();
    const ok = document.execCommand("copy");
    area.remove();
    if (!ok) throw new Error("copie impossible");
  }
}

/** Nouveau secret partagé, au même format que celui créé par Bascule. */
export function newSecret(): string {
  const alphabet = "abcdefghjkmnpqrstuvwxyz23456789";
  const bytes = crypto.getRandomValues(new Uint8Array(24));
  const s = Array.from(bytes, (b) => alphabet[b % alphabet.length]).join("");
  return s.match(/.{4}/g)!.join("-");
}

export const osLabel = (os: string) => ({ macos: "macOS", windows: "Windows", linux: "Linux" })[os] ?? os;

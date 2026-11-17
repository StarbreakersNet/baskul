export async function copyText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    copyWithTextArea(text);
  }
}

// WebKitGTK refuse parfois l'API du presse-papiers : repli sur l'ancienne commande de copie
function copyWithTextArea(text: string) {
  const area = document.createElement("textarea");
  area.value = text;
  area.style.position = "fixed";
  area.style.opacity = "0";
  document.body.append(area);
  area.select();
  const copied = document.execCommand("copy");
  area.remove();
  if (!copied) {
    throw new Error("copie impossible");
  }
}

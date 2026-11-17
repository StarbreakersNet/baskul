/** Adresse à saisir : sans le port quand c'est celui par défaut, et sans crochets IPv6 */
export function hostWithoutDefaultPort(address: string, defaultPort?: number) {
  const suffix = `:${defaultPort}`;
  return defaultPort && address.endsWith(suffix)
    ? address.slice(0, -suffix.length).replace(/^\[(.*)\]$/, "$1")
    : address;
}

/** Commande prête à coller dans un terminal, avec le chemin de l'exécutable entre guillemets si besoin */
export function shellCommand(exe: string, args: string, os: string) {
  let quoted = exe;
  if (/[\s'"()&;]/.test(exe)) {
    quoted = os === "windows" ? `"${exe}"` : `'${exe.replace(/'/g, "'\\''")}'`;
  }
  return `${quoted} ${args}`;
}

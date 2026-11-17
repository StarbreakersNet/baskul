import { REPOSITORY_URL } from "@/models/Repository";

/** Nouvelle issue GitHub, avec un texte déjà rempli (version de Baskul, système) */
export function bugReportUrl(body: string) {
  return `${REPOSITORY_URL}/issues/new?${new URLSearchParams({ body })}`;
}

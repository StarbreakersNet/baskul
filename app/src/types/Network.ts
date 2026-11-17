export interface NetInterface {
  /** Nom donné par le système, enregistré dans la configuration */
  name: string;
  /** Nom lisible : "Wi-Fi", "Ethernet"… */
  label: string;
  addresses: string[];
}

/** Machine principale trouvée sur le réseau local */
export interface FoundPrimary {
  name: string;
  addresses: string[];
  os: string;
  protocol: number;
}

/** Réponse de `network_info` */
export interface NetworkInfo {
  interfaces: NetInterface[];
  primaries: FoundPrimary[];
  /** Version du protocole de cette machine */
  protocol: number;
  defaultPort: number;
}

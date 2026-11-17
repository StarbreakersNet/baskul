import { IconDeviceDesktop, IconDeviceLaptop } from "@tabler/icons-vue";

const OS_LABELS: Record<string, string> = { macos: "macOS", windows: "Windows", linux: "Linux" };

export function osLabel(os: string) {
  return OS_LABELS[os] ?? os;
}

export function osIcon(os?: string) {
  return os === "macos" ? IconDeviceLaptop : IconDeviceDesktop;
}

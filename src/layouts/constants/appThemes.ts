import type { AppTheme } from "../types/settings";

export interface AppThemeOption {
  readonly value: AppTheme;
  readonly label: string;
  readonly description: string;
  readonly previewBackground: string;
}

export const DEFAULT_APP_THEME: AppTheme = "soft-glow";

export const APP_THEME_OPTIONS = [
  {
    value: "soft-glow",
    label: "柔光灰",
    description: "macOS 风格中灰背景，带柔和环境光。",
    previewBackground:
      "radial-gradient(circle at 15% 110%, #d9a6b8, transparent 52%), radial-gradient(circle at 58% 110%, #9ebbd5, transparent 54%), radial-gradient(circle at 100% 110%, #a7c9b8, transparent 50%), #cfd1d3",
  },
  {
    value: "classic",
    label: "经典浅色",
    description: "简洁浅灰背景，搭配清晰的纯白卡片。",
    previewBackground: "linear-gradient(135deg, #f6f7f9 0 58%, #ffffff 58% 100%)",
  },
  {
    value: "dark",
    label: "深色",
    description: "炭黑背景与深灰卡片，适合低光环境。",
    previewBackground: "linear-gradient(135deg, #151719 0 58%, #25282c 58% 100%)",
  },
] as const satisfies readonly AppThemeOption[];

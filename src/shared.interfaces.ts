import { Layout, LayoutSchema, NoFallbackBehavior } from './apps/utils/schemas/Layout';
import { Placeholder } from './apps/utils/schemas/Placeholders';
import { ISettings } from './apps/utils/schemas/Settings';
import { Theme, ThemeSchema } from './apps/utils/schemas/Theme';

export interface IRootState<T> {
  settings: T;
  theme: Theme;
}

export interface UserSettings {
  jsonSettings: ISettings;
  yamlSettings: anyObject[];
  themes: Theme[];
  theme: Theme | null;
  layouts: Layout[];
  placeholders: Placeholder[];
  env: Record<string, string>;
}

export interface AppTemplate {
  name: string;
  description: string;
  apps: anyObject[];
}

const _defaultTheme = ThemeSchema.parse({});
export const defaultTheme: Theme = {
  ..._defaultTheme,
  info: {
    ..._defaultTheme.info,
    cssFileUrl: null,
    filename: 'unknown',
  },
};

const _defaultLayout = LayoutSchema.parse({});
export const defaultLayout: Layout = {
  ..._defaultLayout,
  info: {
    ..._defaultLayout.info,
    filename: 'unknown',
  },
  noFallbackBehavior: NoFallbackBehavior.Unmanaged,
};
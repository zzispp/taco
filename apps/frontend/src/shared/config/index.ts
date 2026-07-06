import { paths } from 'src/shared/routes/paths';

import packageJson from '../../../package.json';

// ----------------------------------------------------------------------

export type ConfigValue = {
  appName: string;
  appVersion: string;
  serverUrl: string;
  assetsDir: string;
  isStaticExport: boolean;
  auth: {
    skip: boolean;
    redirectPath: string;
  };
};

// ----------------------------------------------------------------------

const DEFAULT_SERVER_URL = 'http://127.0.0.1:3000';

export const CONFIG: ConfigValue = {
  appName: 'taco',
  appVersion: packageJson.version,
  serverUrl: process.env.NEXT_PUBLIC_SERVER_URL ?? DEFAULT_SERVER_URL,
  assetsDir: process.env.NEXT_PUBLIC_ASSETS_DIR ?? '',
  isStaticExport: JSON.parse(process.env.BUILD_STATIC_EXPORT ?? 'false'),
  auth: {
    skip: false,
    redirectPath: paths.dashboard.root,
  },
};

import { paths } from 'src/shared/routes/paths';

import packageJson from '../../../package.json';

// ----------------------------------------------------------------------

const SAME_ORIGIN_ASSETS_PREFIX = '';

export type ConfigValue = {
  appName: string;
  appVersion: string;
  assetsDir: string;
  auth: {
    skip: boolean;
    redirectPath: string;
  };
};

// ----------------------------------------------------------------------

export const CONFIG: ConfigValue = {
  appName: 'taco',
  appVersion: packageJson.version,
  assetsDir: SAME_ORIGIN_ASSETS_PREFIX,
  auth: {
    skip: false,
    redirectPath: paths.dashboard.root,
  },
};

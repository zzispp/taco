'use client';

import type { SettingsState, SettingsContextValue } from '../types';

import { useCallback } from 'react';
import { hasKeys } from 'minimal-shared/utils';

import Box from '@mui/material/Box';
import Badge from '@mui/material/Badge';
import SvgIcon from '@mui/material/SvgIcon';
import Tooltip from '@mui/material/Tooltip';
import IconButton from '@mui/material/IconButton';
import Typography from '@mui/material/Typography';
import { useColorScheme } from '@mui/material/styles';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { primaryColorPresets } from 'src/shared/theme/with-settings';

import { Label } from '../../label';
import { settingIcons } from './icons';
import { Iconify } from '../../iconify';
import { BaseOption } from './base-option';
import { SmallBlock, LargeBlock } from './styles';
import { PresetsOptions } from './presets-options';
import { FullScreenButton } from './fullscreen-button';
import { NavColorOptions, NavLayoutOptions } from './nav-layout-option';

export type SettingsVisibility = ReturnType<typeof settingsVisibility>;

type SettingsSectionProps = {
  settings: SettingsContextValue;
  defaultSettings: SettingsState;
  visibility: SettingsVisibility;
};

export function settingsVisibility(defaultSettings: SettingsState) {
  return {
    mode: hasKeys(defaultSettings, ['mode']),
    contrast: hasKeys(defaultSettings, ['contrast']),
    navColor: hasKeys(defaultSettings, ['navColor']),
    fontSize: hasKeys(defaultSettings, ['fontSize']),
    direction: hasKeys(defaultSettings, ['direction']),
    navLayout: hasKeys(defaultSettings, ['navLayout']),
    fontFamily: hasKeys(defaultSettings, ['fontFamily']),
    primaryColor: hasKeys(defaultSettings, ['primaryColor']),
    compactLayout: hasKeys(defaultSettings, ['compactLayout']),
  };
}

export function SettingsDrawerHead({ settings }: { settings: SettingsContextValue }) {
  const { t } = useTranslate('common');
  const { setMode } = useColorScheme();
  const handleReset = useCallback(() => {
    settings.onReset();
    setMode(null);
  }, [setMode, settings]);

  return (
    <Box sx={{ py: 2, pr: 1, pl: 2.5, display: 'flex', alignItems: 'center' }}>
      <Typography variant="h6" sx={{ flexGrow: 1 }}>
        {t('settings.title')}
      </Typography>
      <FullScreenButton />
      <Tooltip title={t('settings.resetAll')}>
        <IconButton onClick={handleReset}>
          <Badge color="error" variant="dot" invisible={!settings.canReset}>
            <Iconify icon="solar:restart-bold" />
          </Badge>
        </IconButton>
      </Tooltip>
      <Tooltip title={t('settings.close')}>
        <IconButton onClick={settings.onCloseDrawer}>
          <Iconify icon="mingcute:close-line" />
        </IconButton>
      </Tooltip>
    </Box>
  );
}

export function SettingsToggleOptions({ settings, visibility }: SettingsSectionProps) {
  const { t } = useTranslate('common');
  const { mode, setMode, colorScheme } = useColorScheme();

  return (
    <Box sx={{ gap: 2, display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)' }}>
      {visibility.mode && (
        <BaseOption
          label={t('settings.mode')}
          selected={settings.state.mode === 'dark'}
          icon={<SvgIcon>{settingIcons.moon}</SvgIcon>}
          action={mode === 'system' ? <SystemModeLabel /> : null}
          onChangeOption={() => {
            setMode(colorScheme === 'light' ? 'dark' : 'light');
            settings.setState({ mode: colorScheme === 'light' ? 'dark' : 'light' });
          }}
        />
      )}
      {visibility.contrast && <ContrastOption settings={settings} />}
      {visibility.direction && <DirectionOption settings={settings} />}
      {visibility.compactLayout && <CompactOption settings={settings} />}
    </Box>
  );
}

function SystemModeLabel() {
  const { t } = useTranslate('common');

  return (
    <Label
      sx={{ height: 20, cursor: 'inherit', borderRadius: '20px', fontWeight: 'fontWeightSemiBold' }}
    >
      {t('settings.system')}
    </Label>
  );
}

function ContrastOption({ settings }: { settings: SettingsContextValue }) {
  const { t } = useTranslate('common');

  return (
    <BaseOption
      label={t('settings.contrast')}
      selected={settings.state.contrast === 'high'}
      icon={<SvgIcon>{settingIcons.contrast}</SvgIcon>}
      onChangeOption={() => {
        settings.setState({ contrast: settings.state.contrast === 'default' ? 'high' : 'default' });
      }}
    />
  );
}

function DirectionOption({ settings }: { settings: SettingsContextValue }) {
  const { t } = useTranslate('common');

  return (
    <BaseOption
      label={t('settings.rightToLeft')}
      selected={settings.state.direction === 'rtl'}
      icon={<SvgIcon>{settingIcons.alignRight}</SvgIcon>}
      onChangeOption={() => {
        settings.setState({ direction: settings.state.direction === 'ltr' ? 'rtl' : 'ltr' });
      }}
    />
  );
}

function CompactOption({ settings }: { settings: SettingsContextValue }) {
  const { t } = useTranslate('common');

  return (
    <BaseOption
      tooltip={t('settings.compactTooltip')}
      label={t('settings.compact')}
      selected={!!settings.state.compactLayout}
      icon={<SvgIcon>{settingIcons.autofitWidth}</SvgIcon>}
      onChangeOption={() => {
        settings.setState({ compactLayout: !settings.state.compactLayout });
      }}
    />
  );
}

export function SettingsPresetOptions({ settings, defaultSettings }: SettingsSectionProps) {
  const { t } = useTranslate('common');

  return (
    <LargeBlock
      title={t('settings.presets')}
      canReset={settings.state.primaryColor !== defaultSettings.primaryColor}
      onReset={() => settings.setState({ primaryColor: defaultSettings.primaryColor })}
    >
      <PresetsOptions
        icon={<SvgIcon sx={{ width: 28, height: 28 }}>{settingIcons.siderbarDuotone}</SvgIcon>}
        options={(Object.keys(primaryColorPresets) as SettingsState['primaryColor'][]).map(
          (key) => ({
            name: key,
            value: primaryColorPresets[key].main,
          })
        )}
        value={settings.state.primaryColor}
        onChangeOption={(newOption) => settings.setState({ primaryColor: newOption })}
      />
    </LargeBlock>
  );
}

export function SettingsNavigationOptions({
  settings,
  defaultSettings,
  visibility,
}: SettingsSectionProps) {
  const { t } = useTranslate('common');

  return (
    <LargeBlock title={t('settings.nav')} tooltip={t('settings.dashboardOnly')} sx={{ gap: 2.5 }}>
      {visibility.navLayout && (
        <SmallBlock
          label={t('settings.layout')}
          canReset={settings.state.navLayout !== defaultSettings.navLayout}
          onReset={() => settings.setState({ navLayout: defaultSettings.navLayout })}
        >
          <NavLayoutOptions
            value={settings.state.navLayout}
            onChangeOption={(newOption) => settings.setState({ navLayout: newOption })}
            options={navLayoutOptions()}
          />
        </SmallBlock>
      )}
      {visibility.navColor && (
        <NavColorBlock settings={settings} defaultSettings={defaultSettings} />
      )}
    </LargeBlock>
  );
}

function navLayoutOptions() {
  return [
    {
      value: 'vertical' as const,
      icon: <SvgIcon sx={{ width: 1, height: 'auto' }}>{settingIcons.navVertical}</SvgIcon>,
    },
    {
      value: 'horizontal' as const,
      icon: <SvgIcon sx={{ width: 1, height: 'auto' }}>{settingIcons.navHorizontal}</SvgIcon>,
    },
    {
      value: 'mini' as const,
      icon: <SvgIcon sx={{ width: 1, height: 'auto' }}>{settingIcons.navMini}</SvgIcon>,
    },
  ];
}

function NavColorBlock({ settings, defaultSettings }: Omit<SettingsSectionProps, 'visibility'>) {
  const { t } = useTranslate('common');

  return (
    <SmallBlock
      label={t('settings.color')}
      canReset={settings.state.navColor !== defaultSettings.navColor}
      onReset={() => settings.setState({ navColor: defaultSettings.navColor })}
    >
      <NavColorOptions
        value={settings.state.navColor}
        onChangeOption={(newOption) => settings.setState({ navColor: newOption })}
        options={[
          {
            label: t('settings.integrate'),
            value: 'integrate',
            icon: <SvgIcon>{settingIcons.sidebarOutline}</SvgIcon>,
          },
          {
            label: t('settings.apparent'),
            value: 'apparent',
            icon: <SvgIcon>{settingIcons.sidebarFill}</SvgIcon>,
          },
        ]}
      />
    </SmallBlock>
  );
}

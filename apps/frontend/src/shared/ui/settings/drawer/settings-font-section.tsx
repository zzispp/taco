'use client';

import type { SettingsState, SettingsContextValue } from '../types';

import SvgIcon from '@mui/material/SvgIcon';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { themeConfig } from 'src/shared/theme/theme-config';

import { settingIcons } from './icons';
import { SmallBlock, LargeBlock } from './styles';
import { FontSizeOptions, FontFamilyOptions } from './font-options';

const FONT_SIZE_RANGE: [number, number] = [12, 20];

type SettingsFontOptionsProps = {
  settings: SettingsContextValue;
  defaultSettings: SettingsState;
  visibility: { fontFamily: boolean; fontSize: boolean };
};

export function SettingsFontOptions({
  settings,
  defaultSettings,
  visibility,
}: SettingsFontOptionsProps) {
  const { t } = useTranslate('common');

  return (
    <LargeBlock title={t('settings.font')} sx={{ gap: 2.5 }}>
      {visibility.fontFamily && (
        <SmallBlock
          label={t('settings.family')}
          canReset={settings.state.fontFamily !== defaultSettings.fontFamily}
          onReset={() => settings.setState({ fontFamily: defaultSettings.fontFamily })}
        >
          <FontFamilyOptions
            value={settings.state.fontFamily}
            onChangeOption={(fontFamily) => settings.setState({ fontFamily })}
            options={fontFamilyOptions()}
            icon={<SvgIcon sx={{ width: 28, height: 28 }}>{settingIcons.font}</SvgIcon>}
          />
        </SmallBlock>
      )}
      {visibility.fontSize && (
        <SmallBlock
          label={t('settings.size')}
          canReset={settings.state.fontSize !== defaultSettings.fontSize}
          onReset={() => settings.setState({ fontSize: defaultSettings.fontSize })}
          sx={{ gap: 5 }}
        >
          <FontSizeOptions
            options={FONT_SIZE_RANGE}
            value={settings.state.fontSize}
            onChangeOption={(fontSize) => settings.setState({ fontSize })}
          />
        </SmallBlock>
      )}
    </LargeBlock>
  );
}

function fontFamilyOptions() {
  return [
    themeConfig.fontFamily.primary,
    'Inter Variable',
    'DM Sans Variable',
    'Nunito Sans Variable',
  ];
}

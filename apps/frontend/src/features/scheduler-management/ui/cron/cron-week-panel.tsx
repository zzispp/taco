import type { WeekMode, WeekFieldState } from '../../model/cron/cron-builder-model';

import Typography from '@mui/material/Typography';

import { MIN_WEEKDAY, MAX_NTH_WEEK, WEEKDAY_VALUES } from '../../model/cron/cron-builder-model';
import {
  CronSelect,
  CronRadioGroup,
  CronMultiSelect,
  CronRadioOption,
  CronNumberInput,
} from './cron-field-controls';

type Translate = (key: string, options?: Record<string, unknown>) => string;

type WeekPanelProps = {
  value: WeekFieldState;
  t: Translate;
  onChange: (patch: Partial<WeekFieldState>) => void;
};

export function CronWeekPanel({ value, t, onChange }: WeekPanelProps) {
  return (
    <CronRadioGroup value={value.mode} onChange={(mode) => onChange({ mode: mode as WeekMode })}>
      <CronRadioOption value="any">
        <Typography>
          {t('cronBuilderUi.allowed', { field: t('cron.week'), chars: '[, - * ? / L #]' })}
        </Typography>
      </CronRadioOption>
      <CronRadioOption value="unspecified">
        <Typography>{t('cronBuilderUi.unspecified')}</Typography>
      </CronRadioOption>
      <WeekRangeOption value={value} t={t} onChange={onChange} />
      <CronRadioOption value="nthWeekday">
        <Typography>{t('cronBuilderUi.nthWeekdayPrefix')}</Typography>
        <CronNumberInput
          label={t('cronBuilderUi.nth')}
          min={MIN_WEEKDAY}
          max={MAX_NTH_WEEK}
          value={value.nth}
          onChange={(nth) => onChange({ mode: 'nthWeekday', nth })}
        />
        <Typography>{t('cronBuilderUi.nthWeekdayMiddle')}</Typography>
        <CronSelect
          label={t('cron.week')}
          options={weekdayOptions(t)}
          value={value.nthWeekday}
          onChange={(nthWeekday) => onChange({ mode: 'nthWeekday', nthWeekday })}
        />
      </CronRadioOption>
      <CronRadioOption value="lastWeekday">
        <Typography>{t('cronBuilderUi.lastWeekdayPrefix')}</Typography>
        <CronSelect
          label={t('cron.week')}
          options={weekdayOptions(t)}
          value={value.lastWeekday}
          onChange={(lastWeekday) => onChange({ mode: 'lastWeekday', lastWeekday })}
        />
      </CronRadioOption>
      <CronRadioOption value="specific">
        <Typography>{t('cronBuilderUi.specific')}</Typography>
        <CronMultiSelect
          label={t('cronBuilderUi.multiPlaceholder')}
          options={weekdayOptions(t)}
          value={value.selected}
          onChange={(selected) => onChange({ mode: 'specific', selected })}
        />
      </CronRadioOption>
    </CronRadioGroup>
  );
}

function WeekRangeOption({ value, t, onChange }: WeekPanelProps) {
  return (
    <CronRadioOption value="range">
      <Typography>{t('cronBuilderUi.weekRange')}</Typography>
      <CronSelect
        label={t('cronBuilderUi.start')}
        options={weekdayOptions(t, undefined, true)}
        value={value.rangeStart}
        onChange={(rangeStart) => onChange({ mode: 'range', rangeStart })}
      />
      <Typography>-</Typography>
      <CronSelect
        label={t('cronBuilderUi.end')}
        options={weekdayOptions(t, value.rangeStart)}
        value={value.rangeEnd}
        onChange={(rangeEnd) => onChange({ mode: 'range', rangeEnd })}
      />
    </CronRadioOption>
  );
}

function weekdayOptions(t: Translate, rangeStart?: number, disableSunday = false) {
  return WEEKDAY_VALUES.map((value) => ({
    label: t(`cronBuilderUi.weekdays.${value}`),
    value: String(value),
    disabled: disableSunday
      ? value === 1
      : rangeStart !== undefined && value < rangeStart && value !== 1,
  }));
}

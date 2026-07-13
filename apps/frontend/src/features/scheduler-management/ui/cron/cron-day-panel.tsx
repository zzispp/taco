import type { DayMode, DayFieldState } from '../../model/cron/cron-builder-model';

import Typography from '@mui/material/Typography';

import { DAY_CONFIG, numberOptions } from '../../model/cron/cron-builder-model';
import {
  CronRadioGroup,
  CronMultiSelect,
  CronRadioOption,
  CronNumberInput,
} from './cron-field-controls';

type Translate = (key: string, options?: Record<string, unknown>) => string;

type DayPanelProps = {
  value: DayFieldState;
  t: Translate;
  onChange: (patch: Partial<DayFieldState>) => void;
};

export function CronDayPanel({ value, t, onChange }: DayPanelProps) {
  const options = numberOptions(DAY_CONFIG.min, DAY_CONFIG.max).map((item) => ({
    label: item,
    value: item,
  }));
  const fieldLabel = t('cron.day');

  return (
    <CronRadioGroup value={value.mode} onChange={(mode) => onChange({ mode: mode as DayMode })}>
      <CronRadioOption value="any">
        <Typography>
          {t('cronBuilderUi.allowed', { field: fieldLabel, chars: '[, - * ? / L W]' })}
        </Typography>
      </CronRadioOption>
      <CronRadioOption value="unspecified">
        <Typography>{t('cronBuilderUi.unspecified')}</Typography>
      </CronRadioOption>
      <DayRangeOption value={value} t={t} onChange={onChange} />
      <DayIntervalOption value={value} t={t} onChange={onChange} />
      <CronRadioOption value="workday">
        <Typography>{t('cronBuilderUi.nearestWorkdayPrefix')}</Typography>
        <CronNumberInput
          label={fieldLabel}
          min={DAY_CONFIG.min}
          max={DAY_CONFIG.max}
          value={value.workday}
          onChange={(workday) => onChange({ mode: 'workday', workday })}
        />
        <Typography>{t('cronBuilderUi.nearestWorkdaySuffix')}</Typography>
      </CronRadioOption>
      <CronRadioOption value="lastDay">
        <Typography>{t('cronBuilderUi.lastDay')}</Typography>
      </CronRadioOption>
      <CronRadioOption value="specific">
        <Typography>{t('cronBuilderUi.specific')}</Typography>
        <CronMultiSelect
          label={t('cronBuilderUi.multiPlaceholder')}
          options={options}
          value={value.selected}
          onChange={(selected) => onChange({ mode: 'specific', selected })}
        />
      </CronRadioOption>
    </CronRadioGroup>
  );
}

function DayRangeOption({ value, t, onChange }: DayPanelProps) {
  return (
    <CronRadioOption value="range">
      <Typography>{t('cronBuilderUi.range')}</Typography>
      <CronNumberInput
        label={t('cronBuilderUi.start')}
        min={DAY_CONFIG.min}
        max={DAY_CONFIG.rangeStartMax}
        value={value.rangeStart}
        onChange={(rangeStart) =>
          onChange({
            mode: 'range',
            rangeStart,
            rangeEnd: Math.max(value.rangeEnd, rangeStart + 1),
          })
        }
      />
      <Typography>-</Typography>
      <CronNumberInput
        label={t('cronBuilderUi.end')}
        min={value.rangeStart + 1}
        max={DAY_CONFIG.max}
        value={value.rangeEnd}
        onChange={(rangeEnd) => onChange({ mode: 'range', rangeEnd })}
      />
      <Typography>{t('cron.day')}</Typography>
    </CronRadioOption>
  );
}

function DayIntervalOption({ value, t, onChange }: DayPanelProps) {
  const maxStep = Math.max(1, DAY_CONFIG.max - value.intervalStart);
  return (
    <CronRadioOption value="interval">
      <Typography>{t('cronBuilderUi.from')}</Typography>
      <CronNumberInput
        label={t('cron.day')}
        min={DAY_CONFIG.min}
        max={DAY_CONFIG.rangeStartMax}
        value={value.intervalStart}
        onChange={(intervalStart) =>
          onChange({
            mode: 'interval',
            intervalStart,
            intervalStep: Math.min(value.intervalStep, DAY_CONFIG.max - intervalStart),
          })
        }
      />
      <Typography>{t('cronBuilderUi.intervalEvery')}</Typography>
      <CronNumberInput
        label={t('cronBuilderUi.step')}
        min={1}
        max={maxStep}
        value={value.intervalStep}
        onChange={(intervalStep) => onChange({ mode: 'interval', intervalStep })}
      />
      <Typography>{t('cronBuilderUi.executeOnce')}</Typography>
    </CronRadioOption>
  );
}

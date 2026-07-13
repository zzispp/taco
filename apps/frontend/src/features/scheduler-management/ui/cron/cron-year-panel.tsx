import type { YearMode, YearFieldState } from '../../model/cron/cron-builder-model';

import Typography from '@mui/material/Typography';

import { MAX_YEAR, yearOptions, MAX_YEAR_RANGE_START } from '../../model/cron/cron-builder-model';
import {
  CronRadioGroup,
  CronMultiSelect,
  CronRadioOption,
  CronNumberInput,
} from './cron-field-controls';

type Translate = (key: string, options?: Record<string, unknown>) => string;

type YearPanelProps = {
  value: YearFieldState;
  t: Translate;
  currentYear: number;
  onChange: (patch: Partial<YearFieldState>) => void;
};

export function CronYearPanel({ value, t, currentYear, onChange }: YearPanelProps) {
  const options = yearOptions(currentYear).map((item) => ({ label: item, value: item }));

  return (
    <CronRadioGroup value={value.mode} onChange={(mode) => onChange({ mode: mode as YearMode })}>
      <CronRadioOption value="blank">
        <Typography>{t('cronBuilderUi.yearBlankAllowed')}</Typography>
      </CronRadioOption>
      <CronRadioOption value="yearly">
        <Typography>{t('cronBuilderUi.yearly')}</Typography>
      </CronRadioOption>
      <YearRangeOption value={value} t={t} currentYear={currentYear} onChange={onChange} />
      <YearIntervalOption value={value} t={t} currentYear={currentYear} onChange={onChange} />
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

function YearRangeOption({ value, t, currentYear, onChange }: YearPanelProps) {
  return (
    <CronRadioOption value="range">
      <Typography>{t('cronBuilderUi.range')}</Typography>
      <CronNumberInput
        label={t('cronBuilderUi.start')}
        min={currentYear}
        max={MAX_YEAR_RANGE_START}
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
        max={MAX_YEAR}
        value={value.rangeEnd}
        onChange={(rangeEnd) => onChange({ mode: 'range', rangeEnd })}
      />
    </CronRadioOption>
  );
}

function YearIntervalOption({ value, t, currentYear, onChange }: YearPanelProps) {
  const maxStep = Math.max(1, MAX_YEAR - value.intervalStart);
  return (
    <CronRadioOption value="interval">
      <Typography>{t('cronBuilderUi.from')}</Typography>
      <CronNumberInput
        label={t('cron.year')}
        min={currentYear}
        max={MAX_YEAR_RANGE_START}
        value={value.intervalStart}
        onChange={(intervalStart) =>
          onChange({
            mode: 'interval',
            intervalStart,
            intervalStep: Math.min(value.intervalStep, MAX_YEAR - intervalStart),
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

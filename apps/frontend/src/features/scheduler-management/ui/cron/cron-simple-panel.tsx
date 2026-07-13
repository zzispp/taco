import type {
  SimpleMode,
  SimpleCronField,
  SimpleFieldState,
} from '../../model/cron/cron-builder-model';

import Typography from '@mui/material/Typography';

import { numberOptions, SIMPLE_FIELD_CONFIG } from '../../model/cron/cron-builder-model';
import {
  CronRadioGroup,
  CronMultiSelect,
  CronRadioOption,
  CronNumberInput,
} from './cron-field-controls';

type Translate = (key: string, options?: Record<string, unknown>) => string;

type SimplePanelProps = {
  field: SimpleCronField;
  value: SimpleFieldState;
  t: Translate;
  onChange: (patch: Partial<SimpleFieldState>) => void;
};

type SimpleOptionProps = SimplePanelProps & {
  fieldLabel: string;
};

export function CronSimplePanel(props: SimplePanelProps) {
  const { field, value, t, onChange } = props;
  const fieldLabel = t(`cron.${field}`);

  return (
    <CronRadioGroup value={value.mode} onChange={(mode) => onChange({ mode: mode as SimpleMode })}>
      <CronRadioOption value="any">
        <Typography>
          {t('cronBuilderUi.allowed', { field: fieldLabel, chars: '[, - * /]' })}
        </Typography>
      </CronRadioOption>
      <SimpleRangeOption {...props} fieldLabel={fieldLabel} />
      <SimpleIntervalOption {...props} fieldLabel={fieldLabel} />
      <SimpleSpecificOption {...props} fieldLabel={fieldLabel} />
    </CronRadioGroup>
  );
}

function SimpleRangeOption({ field, value, t, onChange, fieldLabel }: SimpleOptionProps) {
  const config = SIMPLE_FIELD_CONFIG[field];
  return (
    <CronRadioOption value="range">
      <Typography>{t('cronBuilderUi.range')}</Typography>
      <CronNumberInput
        label={t('cronBuilderUi.start')}
        min={config.min}
        max={config.rangeStartMax}
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
        max={config.max}
        value={value.rangeEnd}
        onChange={(rangeEnd) => onChange({ mode: 'range', rangeEnd })}
      />
      <Typography>{fieldLabel}</Typography>
    </CronRadioOption>
  );
}

function SimpleIntervalOption({ field, value, t, onChange, fieldLabel }: SimpleOptionProps) {
  const config = SIMPLE_FIELD_CONFIG[field];
  const maxStep = Math.max(1, config.max - value.intervalStart);
  return (
    <CronRadioOption value="interval">
      <Typography>{t('cronBuilderUi.from')}</Typography>
      <CronNumberInput
        label={fieldLabel}
        min={config.min}
        max={config.rangeStartMax}
        value={value.intervalStart}
        onChange={(intervalStart) =>
          onChange({
            mode: 'interval',
            intervalStart,
            intervalStep: Math.min(value.intervalStep, config.max - intervalStart),
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

function SimpleSpecificOption({ field, value, t, onChange }: SimpleOptionProps) {
  const config = SIMPLE_FIELD_CONFIG[field];
  const options = numberOptions(config.min, config.max).map((item) => ({
    label: item,
    value: item,
  }));
  return (
    <CronRadioOption value="specific">
      <Typography>{t('cronBuilderUi.specific')}</Typography>
      <CronMultiSelect
        label={t('cronBuilderUi.multiPlaceholder')}
        options={options}
        value={value.selected}
        onChange={(selected) => onChange({ mode: 'specific', selected })}
      />
    </CronRadioOption>
  );
}

import type { Dispatch, SetStateAction } from 'react';
import type { CronField, CronBuilderState } from '../../model/cron/cron-builder-model';

import { CronDayPanel } from './cron-day-panel';
import { CronWeekPanel } from './cron-week-panel';
import { CronYearPanel } from './cron-year-panel';
import { CronSimplePanel } from './cron-simple-panel';
import {
  updateDayField,
  updateWeekField,
  updateYearField,
  updateSimpleField,
} from '../../model/cron/cron-builder-model';

type Translate = (key: string, options?: Record<string, unknown>) => string;

type CronFieldPanelsProps = {
  activeField: CronField;
  currentYear: number;
  state: CronBuilderState;
  t: Translate;
  setState: Dispatch<SetStateAction<CronBuilderState>>;
};

export function CronFieldPanels({
  activeField,
  currentYear,
  state,
  t,
  setState,
}: CronFieldPanelsProps) {
  if (activeField === 'day')
    return (
      <CronDayPanel
        value={state.day}
        t={t}
        onChange={(patch) => setState((current) => updateDayField(current, patch))}
      />
    );
  if (activeField === 'week')
    return (
      <CronWeekPanel
        value={state.week}
        t={t}
        onChange={(patch) => setState((current) => updateWeekField(current, patch))}
      />
    );
  if (activeField === 'year')
    return (
      <CronYearPanel
        value={state.year}
        t={t}
        currentYear={currentYear}
        onChange={(patch) => setState((current) => updateYearField(current, patch))}
      />
    );

  return (
    <CronSimplePanel
      field={activeField}
      value={state[activeField]}
      t={t}
      onChange={(patch) => setState((current) => updateSimpleField(current, activeField, patch))}
    />
  );
}

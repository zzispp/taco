import type { Dispatch, SetStateAction } from 'react';
import type { JobFormState } from '../model/job-form';
import type { TaskParamFormSpec } from 'src/entities/scheduler';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';

import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  MISFIRE_POLICY,
  CONCURRENT_POLICY,
  misfirePolicyTranslationKeys,
  concurrentPolicyTranslationKeys,
} from 'src/entities/scheduler';

import { ParamFields } from './param-fields';

type JobFormFieldsProps = {
  form: JobFormState;
  paramForm: TaskParamFormSpec;
  setForm: Dispatch<SetStateAction<JobFormState>>;
  openCron: () => void;
};

export function JobFormFields(props: JobFormFieldsProps) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const write = useFormWriter(props.setForm);
  return (
    <Stack spacing={2.5} sx={{ pt: 1 }}>
      <TextField label={t('taskKey')} value={props.form.task_key} disabled />
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextField
          fullWidth
          label={t('jobName')}
          value={props.form.job_name}
          onChange={(event) => write('job_name', event.target.value)}
        />
        <TextField
          fullWidth
          label={t('jobGroup')}
          value={props.form.job_group}
          onChange={(event) => write('job_group', event.target.value)}
        />
      </Stack>
      <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
        <TextField
          fullWidth
          label={t('cronExpression')}
          value={props.form.cron_expression}
          onChange={(event) => write('cron_expression', event.target.value)}
        />
        <Button variant="outlined" onClick={props.openCron}>
          {t('cronBuilder')}
        </Button>
      </Stack>
      <PolicyFields form={props.form} write={write} />
      <ParamFields
        fields={props.paramForm.ui.fields}
        draft={props.form.paramDraft}
        onChange={(paramDraft) => write('paramDraft', paramDraft)}
      />
      <TextField
        multiline
        minRows={2}
        label={tAdmin('common.remark')}
        value={props.form.remark ?? ''}
        onChange={(event) => write('remark', event.target.value)}
      />
    </Stack>
  );
}

type FormWriter = <K extends keyof JobFormState>(key: K, value: JobFormState[K]) => void;

function PolicyFields({ form, write }: { form: JobFormState; write: FormWriter }) {
  const { t } = useTranslate('scheduler');
  return (
    <Stack direction={{ xs: 'column', md: 'row' }} spacing={2}>
      <TextField
        select
        fullWidth
        label={t('misfirePolicy')}
        value={form.misfire_policy}
        onChange={(event) =>
          write('misfire_policy', event.target.value as JobFormState['misfire_policy'])
        }
      >
        {Object.values(MISFIRE_POLICY).map((value) => (
          <MenuItem key={value} value={value}>
            {t(misfirePolicyTranslationKeys[value])}
          </MenuItem>
        ))}
      </TextField>
      <TextField
        select
        fullWidth
        label={t('concurrent')}
        value={form.concurrent}
        onChange={(event) => write('concurrent', event.target.value as JobFormState['concurrent'])}
      >
        {Object.values(CONCURRENT_POLICY).map((value) => (
          <MenuItem key={value} value={value}>
            {t(concurrentPolicyTranslationKeys[value])}
          </MenuItem>
        ))}
      </TextField>
    </Stack>
  );
}

function useFormWriter(setForm: Dispatch<SetStateAction<JobFormState>>): FormWriter {
  return (key, value) => setForm((current) => ({ ...current, [key]: value }));
}

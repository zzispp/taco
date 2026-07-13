import type { ReactNode } from 'react';
import type { JobDetailTab } from '../model/job-detail';
import type { SchedulerJob } from 'src/entities/scheduler';

import Stack from '@mui/material/Stack';

import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  jobStatusTranslationKeys,
  runtimeErrorTranslationKeys,
  misfirePolicyTranslationKeys,
  registryStatusTranslationKeys,
  concurrentPolicyTranslationKeys,
} from 'src/entities/scheduler';

import { DetailField } from './detail-field';
import { RawDetail } from './execution-detail-values';
import {
  JOB_DETAIL_TAB,
  formatTaskParameters,
  jobDetailDisplayValue,
  EMPTY_JOB_DETAIL_VALUE,
  formatRuntimeErrorDetail,
} from '../model/job-detail';

export function JobDetailPanel(props: { job: SchedulerJob; tab: JobDetailTab }) {
  if (props.tab === JOB_DETAIL_TAB.CONFIGURATION) {
    return <ConfigurationPanel job={props.job} />;
  }
  if (props.tab === JOB_DETAIL_TAB.SCHEDULE) return <SchedulePanel job={props.job} />;
  if (props.tab === JOB_DETAIL_TAB.METHOD) return <MethodPanel job={props.job} />;
  return <MetadataPanel job={props.job} />;
}

function ConfigurationPanel({ job }: { job: SchedulerJob }) {
  const { t } = useTranslate('scheduler');
  return (
    <FieldList>
      <DetailField label={t('jobDetail.fields.jobId')}>{job.job_id}</DetailField>
      <DetailField label={t('jobName')}>{job.job_name}</DetailField>
      <DetailField label={t('jobGroup')}>{job.job_group}</DetailField>
      <DetailField label={t('admin:common.status')}>
        {t(jobStatusTranslationKeys[job.status])}
      </DetailField>
      <DetailField label={t('registryStatus')}>
        {t(registryStatusTranslationKeys[job.registry_status])}
      </DetailField>
      <DetailField label={t('jobDetail.fields.repeatable')}>
        {t(job.repeatable ? 'admin:common.yes' : 'admin:common.no')}
      </DetailField>
      <DetailField label={t('jobDetail.fields.scheduleRevision')}>
        {job.schedule_revision}
      </DetailField>
      <DetailField label={t('admin:common.remark')}>
        {jobDetailDisplayValue(job.remark)}
      </DetailField>
    </FieldList>
  );
}

function SchedulePanel({ job }: { job: SchedulerJob }) {
  const { t } = useTranslate('scheduler');
  const runtimeError = job.runtime_error;
  return (
    <FieldList>
      <DetailField label={t('cronExpression')}>{job.cron_expression}</DetailField>
      <DetailField label={t('jobDetail.fields.nextRunAt')}>
        {dateTimeValue(job.next_run_at)}
      </DetailField>
      <DetailField label={t('misfirePolicy')}>
        {t(misfirePolicyTranslationKeys[job.misfire_policy])}
      </DetailField>
      <DetailField label={t('concurrent')}>
        {t(concurrentPolicyTranslationKeys[job.concurrent])}
      </DetailField>
      <DetailField label={t('runtimeError')}>
        {runtimeError
          ? formatRuntimeErrorDetail(
              t(runtimeErrorTranslationKeys[runtimeError.code]),
              runtimeError.message
            )
          : EMPTY_JOB_DETAIL_VALUE}
      </DetailField>
      <DetailField label={t('jobDetail.fields.runtimeErrorAt')}>
        {dateTimeValue(runtimeError?.occurred_at ?? null)}
      </DetailField>
    </FieldList>
  );
}

function MethodPanel({ job }: { job: SchedulerJob }) {
  const { t } = useTranslate('scheduler');
  return (
    <FieldList>
      <DetailField label={t('taskKey')}>{job.task_key}</DetailField>
      <DetailField label={t('invokeTarget')}>{job.invoke_target}</DetailField>
      <DetailField label={t('jobDetail.fields.paramsSchemaVersion')}>
        {job.params_schema_version}
      </DetailField>
      <DetailField label={t('jobDetail.fields.taskParameters')}>
        <RawDetail value={formatTaskParameters(job.task_params)} />
      </DetailField>
    </FieldList>
  );
}

function MetadataPanel({ job }: { job: SchedulerJob }) {
  const { t } = useTranslate('scheduler');
  return (
    <FieldList>
      <DetailField label={t('jobDetail.fields.createdBy')}>
        {jobDetailDisplayValue(job.create_by)}
      </DetailField>
      <DetailField label={t('jobDetail.fields.createdAt')}>
        {dateTimeValue(job.create_time)}
      </DetailField>
      <DetailField label={t('jobDetail.fields.updatedBy')}>
        {jobDetailDisplayValue(job.update_by)}
      </DetailField>
      <DetailField label={t('jobDetail.fields.updatedAt')}>
        {dateTimeValue(job.update_time)}
      </DetailField>
    </FieldList>
  );
}

function FieldList({ children }: { children: ReactNode }) {
  return (
    <Stack spacing={0} component="dl" sx={{ m: 0 }}>
      {children}
    </Stack>
  );
}

function dateTimeValue(value: string | null) {
  return jobDetailDisplayValue(value ? fAdminDateTime(value) : null);
}

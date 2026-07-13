import type { ExecutionDetailTab } from '../model/execution-detail';
import type { SchedulerExecutionDetail } from 'src/entities/scheduler';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';

import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  jobLogStatusTranslationKeys,
  schedulerTriggerTranslationKeys,
  httpExecutionFailureTranslationKeys,
  HTTP_EXECUTION_DETAIL_SCHEMA_VERSION,
} from 'src/entities/scheduler';

import { DetailField } from './detail-field';
import { RawDetail, HttpRequestDetail, HttpResponseDetail } from './execution-detail-values';
import {
  executionDetailView,
  EXECUTION_DETAIL_TAB,
  EXECUTION_DETAIL_VIEW_KIND,
} from '../model/execution-detail';

export function ExecutionDetailPanel(props: {
  detail: SchedulerExecutionDetail;
  tab: ExecutionDetailTab;
}) {
  if (props.tab === EXECUTION_DETAIL_TAB.OVERVIEW) return <Overview detail={props.detail} />;
  if (props.tab === EXECUTION_DETAIL_TAB.PARAMETERS) {
    return <RawDetail value={props.detail.task_params} />;
  }
  const view = executionDetailView(props.detail.detail);
  if (view.kind === EXECUTION_DETAIL_VIEW_KIND.LEGACY) return <LegacyDetail />;
  if (view.kind === EXECUTION_DETAIL_VIEW_KIND.UNKNOWN) return <RawDetail value={view.raw} />;
  return props.tab === EXECUTION_DETAIL_TAB.REQUEST ? (
    <HttpRequestDetail request={view.payload.request} />
  ) : (
    <HttpResponseDetail failure={view.payload.failure} response={view.payload.response} />
  );
}

function Overview({ detail }: { detail: SchedulerExecutionDetail }) {
  return (
    <Stack spacing={0} component="dl" sx={{ m: 0 }}>
      <ExecutionSummary detail={detail} />
      <ExecutionMetadata detail={detail} />
    </Stack>
  );
}

function ExecutionSummary({ detail }: { detail: SchedulerExecutionDetail }) {
  const { t } = useTranslate('scheduler');
  return (
    <>
      <DetailField label={t('executionId')}>{detail.execution_id}</DetailField>
      <DetailField label={t('jobName')}>{detail.job_name}</DetailField>
      <DetailField label={t('jobGroup')}>{detail.job_group}</DetailField>
      <DetailField label={t('taskKey')}>{detail.task_key}</DetailField>
      <DetailField label={t('invokeTarget')}>{detail.invoke_target}</DetailField>
      <DetailField label={t('executionDetail.fields.jobRevision')}>
        {detail.job_revision}
      </DetailField>
      <DetailField label={t('executionDetail.fields.requestedBy')}>
        {detail.requested_by ?? t('executionDetail.notAvailable')}
      </DetailField>
      <DetailField label={t('triggerType')}>
        {t(schedulerTriggerTranslationKeys[detail.trigger_type])}
      </DetailField>
      <DetailField label={t('admin:common.status')}>
        {t(jobLogStatusTranslationKeys[detail.status])}
      </DetailField>
      <DetailField label={t('scheduledAt')}>{fAdminDateTime(detail.scheduled_at)}</DetailField>
      <DetailField label={t('startTime')}>
        {detail.start_time ? fAdminDateTime(detail.start_time) : t('notStarted')}
      </DetailField>
      <DetailField label={t('endTime')}>{fAdminDateTime(detail.end_time)}</DetailField>
      <DetailField label={t('jobMessage')}>{detail.job_message}</DetailField>
      <DetailField label={t('executionDetail.fields.exceptionInfo')}>
        {detail.exception_info ?? t('executionDetail.none')}
      </DetailField>
      <DetailField label={t('executionDetail.fields.createdAt')}>
        {fAdminDateTime(detail.create_time)}
      </DetailField>
    </>
  );
}

function ExecutionMetadata({ detail }: { detail: SchedulerExecutionDetail }) {
  const { t } = useTranslate('scheduler');
  const metadata = detailMetadata(executionDetailView(detail.detail), t);
  return (
    <>
      <DetailField label={t('executionDetail.fields.detailKind')}>{metadata.kind}</DetailField>
      <DetailField label={t('executionDetail.fields.schemaVersion')}>
        {metadata.schemaVersion}
      </DetailField>
      <DetailField label={t('executionDetail.fields.duration')}>{metadata.duration}</DetailField>
      <DetailField label={t('executionDetail.fields.failure')}>{metadata.failure}</DetailField>
    </>
  );
}

type Translate = ReturnType<typeof useTranslate>['t'];

function detailMetadata(view: ReturnType<typeof executionDetailView>, t: Translate) {
  if (view.kind === EXECUTION_DETAIL_VIEW_KIND.LEGACY) {
    const none = t('executionDetail.notAvailable');
    return {
      kind: t('executionDetail.legacy'),
      schemaVersion: none,
      duration: none,
      failure: none,
    };
  }
  if (view.kind === EXECUTION_DETAIL_VIEW_KIND.UNKNOWN) {
    const none = t('executionDetail.notAvailable');
    return {
      kind: `${t('executionDetail.unknown')}: ${view.detail.kind}`,
      schemaVersion: view.detail.schema_version,
      duration: none,
      failure: none,
    };
  }
  return {
    kind: t('executionDetail.httpExchange'),
    schemaVersion: HTTP_EXECUTION_DETAIL_SCHEMA_VERSION,
    duration: t('executionDetail.durationMs', { value: view.payload.duration_ms }),
    failure: view.payload.failure
      ? t(httpExecutionFailureTranslationKeys[view.payload.failure.code])
      : t('executionDetail.none'),
  };
}

function LegacyDetail() {
  const { t } = useTranslate('scheduler');
  return <Alert severity="info">{t('executionDetail.legacyDescription')}</Alert>;
}

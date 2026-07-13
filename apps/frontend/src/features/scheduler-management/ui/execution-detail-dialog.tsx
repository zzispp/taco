'use client';

import type { JobLogController } from '../model/use-job-log-controller';

import { useState } from 'react';

import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import CircularProgress from '@mui/material/CircularProgress';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import { ExecutionDetailPanel } from './execution-detail-panels';
import { EXECUTION_DETAIL_TAB, type ExecutionDetailTab } from '../model/execution-detail';

const DETAIL_CONTENT_MIN_HEIGHT = 520;

const DETAIL_TABS = [
  EXECUTION_DETAIL_TAB.OVERVIEW,
  EXECUTION_DETAIL_TAB.PARAMETERS,
  EXECUTION_DETAIL_TAB.REQUEST,
  EXECUTION_DETAIL_TAB.RESPONSE,
] as const;

export function ExecutionDetailDialog({ controller }: { controller: JobLogController }) {
  const { t } = useTranslate('scheduler');
  const { detail, actions } = controller;
  const [tab, setTab] = useState<ExecutionDetailTab>(EXECUTION_DETAIL_TAB.OVERVIEW);
  const close = () => {
    setTab(EXECUTION_DETAIL_TAB.OVERVIEW);
    actions.closeDetail();
  };
  return (
    <Dialog fullWidth maxWidth="lg" open={Boolean(detail.target)} onClose={close}>
      <DialogTitle>{t('executionDetail.title')}</DialogTitle>
      <DialogContent sx={{ minHeight: DETAIL_CONTENT_MIN_HEIGHT }}>
        <Tabs
          value={tab}
          variant="scrollable"
          onChange={(_, value: ExecutionDetailTab) => setTab(value)}
          sx={{ mb: 3 }}
        >
          {DETAIL_TABS.map((value) => (
            <Tab key={value} label={t(`executionDetail.tabs.${value}`)} value={value} />
          ))}
        </Tabs>
        <DetailContent controller={controller} tab={tab} />
      </DialogContent>
      <DialogActions>
        <Button onClick={close}>{t('admin:common.close')}</Button>
      </DialogActions>
    </Dialog>
  );
}

function DetailContent(props: { controller: JobLogController; tab: ExecutionDetailTab }) {
  const { detail } = props.controller;
  if (detail.loading) {
    return (
      <Box sx={{ py: 10, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress />
      </Box>
    );
  }
  if (detail.error) return <Alert severity="error">{getErrorMessage(detail.error)}</Alert>;
  if (!detail.data || detail.data.execution_id !== detail.target?.execution_id) return null;
  return <ExecutionDetailPanel detail={detail.data} tab={props.tab} />;
}

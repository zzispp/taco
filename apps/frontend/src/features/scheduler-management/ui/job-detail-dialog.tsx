'use client';

import type { JobDetailTab } from '../model/job-detail';
import type { SchedulerController } from '../model/use-scheduler-controller';

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

import { JobDetailPanel } from './job-detail-panels';
import { JOB_DETAIL_TAB, isCurrentJobDetail } from '../model/job-detail';

const DETAIL_CONTENT_MIN_HEIGHT = 520;
const DETAIL_TABS = Object.values(JOB_DETAIL_TAB);

export function JobDetailDialog({ controller }: { controller: SchedulerController }) {
  const { t } = useTranslate('scheduler');
  const [tab, setTab] = useState<JobDetailTab>(JOB_DETAIL_TAB.CONFIGURATION);
  const close = () => {
    setTab(JOB_DETAIL_TAB.CONFIGURATION);
    controller.actions.closeDetail();
  };
  return (
    <Dialog fullWidth maxWidth="lg" open={Boolean(controller.detail.target)} onClose={close}>
      <DialogTitle>{t('jobDetail.title')}</DialogTitle>
      <DialogContent sx={{ minHeight: DETAIL_CONTENT_MIN_HEIGHT }}>
        <Tabs
          value={tab}
          variant="scrollable"
          onChange={(_, value: JobDetailTab) => setTab(value)}
          sx={{ mb: 3 }}
        >
          {DETAIL_TABS.map((value) => (
            <Tab key={value} label={t(`jobDetail.tabs.${value}`)} value={value} />
          ))}
        </Tabs>
        <JobDetailContent controller={controller} tab={tab} />
      </DialogContent>
      <DialogActions>
        <Button onClick={close}>{t('admin:common.close')}</Button>
      </DialogActions>
    </Dialog>
  );
}

function JobDetailContent(props: { controller: SchedulerController; tab: JobDetailTab }) {
  const { detail } = props.controller;
  if (detail.loading) {
    return (
      <Box sx={{ py: 10, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress />
      </Box>
    );
  }
  if (detail.error) return <Alert severity="error">{getErrorMessage(detail.error)}</Alert>;
  if (!isCurrentJobDetail(detail.target, detail.data)) return null;
  return <JobDetailPanel job={detail.data} tab={props.tab} />;
}

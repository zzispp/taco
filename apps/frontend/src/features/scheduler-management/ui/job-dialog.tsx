'use client';

import type { SchedulerJob, ImportableTask } from 'src/entities/scheduler';

import { ZodError } from 'zod';
import { useRef, useMemo, useState, useEffect, useCallback } from 'react';

import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { useHasPermission } from 'src/entities/session';
import { SCHEDULER_PERMISSION } from 'src/entities/scheduler';

import { updateJob, importJob } from '../api';
import { CronDialog } from './cron/cron-dialog';
import { JobFormFields } from './job-form-fields';
import { canPreviewCron } from '../model/permissions';
import { ParamDraftError } from '../model/param-draft';
import { schedulerMutationErrorMessage } from '../model/mutation-error';
import {
  createJobForm,
  emptyParamForm,
  toReplaceJobInput,
  materializeJobInput,
} from '../model/job-form';

type JobDialogProps = {
  open: boolean;
  task?: ImportableTask | null;
  job?: SchedulerJob | null;
  onClose: () => void;
  onSaved: () => void;
};

const createDraftId = () => crypto.randomUUID();

export function JobDialog(props: JobDialogProps) {
  const controller = useJobDialogController(props);
  return (
    <>
      <JobDialogFrame props={props} controller={controller} />
      <JobCronDialog controller={controller} />
    </>
  );
}

function useJobDialogController(props: JobDialogProps) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  const canImport = useHasPermission(SCHEDULER_PERMISSION.JOB_IMPORT);
  const canEdit = useHasPermission(SCHEDULER_PERMISSION.JOB_EDIT);
  const [form, setForm] = useState(() => createJobForm(props.task, props.job, createDraftId));
  const [cronOpen, setCronOpen] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const submittingRef = useRef(false);
  const paramForm = useMemo(
    () => props.task?.param_form ?? props.job?.param_form ?? emptyParamForm(),
    [props.job, props.task]
  );

  useEffect(() => {
    if (props.open) setForm(createJobForm(props.task, props.job, createDraftId));
  }, [props.job, props.open, props.task]);

  const submit = useCallback(async () => {
    if (submittingRef.current) return;
    submittingRef.current = true;
    setSubmitting(true);
    try {
      const input = materializeJobInput(form, paramForm);
      const cacheCapabilities = { canRefreshImportableTasks: canImport };
      if (props.job) {
        await updateJob(props.job.job_id, toReplaceJobInput(input), cacheCapabilities);
      } else {
        await importJob(input, cacheCapabilities);
      }
      toast.success(tAdmin('messages.saved'));
      props.onSaved();
    } catch (error) {
      toast.error(jobFormError(error, t, t('mutation.saveFailed')));
    } finally {
      submittingRef.current = false;
      setSubmitting(false);
    }
  }, [canImport, form, paramForm, props, t, tAdmin]);

  return {
    t,
    tAdmin,
    form,
    setForm,
    cronOpen,
    setCronOpen,
    submitting,
    paramForm,
    canPreview: canPreviewCron({ canImport, canEdit }),
    submit,
  };
}

type JobDialogController = ReturnType<typeof useJobDialogController>;

function JobDialogFrame({
  props,
  controller,
}: {
  props: JobDialogProps;
  controller: JobDialogController;
}) {
  return (
    <Dialog
      fullWidth
      maxWidth="md"
      open={props.open}
      onClose={controller.submitting ? undefined : props.onClose}
    >
      <DialogTitle>{props.job ? controller.t('editJob') : controller.t('importJob')}</DialogTitle>
      <DialogContent>
        <JobFormFields
          form={controller.form}
          paramForm={controller.paramForm}
          setForm={controller.setForm}
          openCron={() => controller.setCronOpen(true)}
        />
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" disabled={controller.submitting} onClick={props.onClose}>
          {controller.tAdmin('common.cancel')}
        </Button>
        <Button variant="contained" loading={controller.submitting} onClick={controller.submit}>
          {controller.tAdmin('common.save')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function JobCronDialog({ controller }: { controller: JobDialogController }) {
  return (
    <CronDialog
      open={controller.cronOpen}
      expression={controller.form.cron_expression}
      canPreview={controller.canPreview}
      onClose={() => controller.setCronOpen(false)}
      onApply={(value) => {
        controller.setForm((current) => ({ ...current, cron_expression: value }));
        controller.setCronOpen(false);
      }}
    />
  );
}

function jobFormError(error: unknown, t: (key: string) => string, fallback: string) {
  if (error instanceof ParamDraftError) {
    const key = {
      invalid_json: 'paramErrors.invalidJson',
      empty_key: 'paramErrors.emptyKey',
      duplicate_key: 'paramErrors.duplicateKey',
      invalid_key_value: 'paramErrors.invalidKeyValue',
    }[error.code];
    return t(key);
  }
  if (error instanceof ZodError) return t('paramErrors.invalidValue');
  return schedulerMutationErrorMessage(error, fallback);
}

'use client';

import type { Notice, NoticeType, NoticeInput, NoticeStatus } from 'src/entities/notice';

import { useState, useEffect } from 'react';

import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Dialog from '@mui/material/Dialog';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import CircularProgress from '@mui/material/CircularProgress';

import { Markdown } from 'src/shared/ui/markdown';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import {
  NOTICE_TYPE,
  NOTICE_STATUS,
  noticeTypeTranslationKeys,
  noticeStatusTranslationKeys,
} from 'src/entities/notice';

import {
  noticeTitleError,
  normalizedNoticeInput,
  noticeInputFromEntity,
  NOTICE_TITLE_MAX_LENGTH,
} from '../model/form';

type NoticeFormDialogProps = Readonly<{
  open: boolean;
  notice?: Notice;
  editing: boolean;
  loading: boolean;
  error?: unknown;
  submitting: boolean;
  onClose: () => void;
  onSubmit: (input: NoticeInput) => void;
}>;

export function NoticeFormDialog(props: NoticeFormDialogProps) {
  const { t } = useTranslate('admin');
  const [form, setForm] = useState<NoticeInput>(() => noticeInputFromEntity(props.notice));
  const [tab, setTab] = useState<'edit' | 'preview'>('edit');
  const [attempted, setAttempted] = useState(false);

  useEffect(() => {
    if (!props.open) return;
    setForm(noticeInputFromEntity(props.notice));
    setTab('edit');
    setAttempted(false);
  }, [props.notice, props.open]);

  const submit = () => {
    setAttempted(true);
    if (props.loading || props.error) return;
    if (noticeTitleError(form)) return;
    props.onSubmit(normalizedNoticeInput(form));
  };

  return (
    <Dialog
      fullWidth
      maxWidth="md"
      open={props.open}
      onClose={props.submitting ? undefined : props.onClose}
    >
      <DialogTitle>{t(props.editing ? 'notice.editTitle' : 'notice.createTitle')}</DialogTitle>
      <DialogContent>
        <NoticeFormContent
          form={form}
          setForm={setForm}
          tab={tab}
          setTab={setTab}
          attempted={attempted}
          loading={props.loading}
          error={props.error}
        />
      </DialogContent>
      <NoticeFormActions dialog={props} onSubmit={submit} />
    </Dialog>
  );
}

function NoticeFormActions({
  dialog,
  onSubmit,
}: {
  dialog: NoticeFormDialogProps;
  onSubmit: () => void;
}) {
  const { t } = useTranslate('admin');
  return (
    <DialogActions>
      <Button variant="outlined" disabled={dialog.submitting} onClick={dialog.onClose}>
        {t('common.cancel')}
      </Button>
      <Button
        variant="contained"
        loading={dialog.submitting}
        disabled={dialog.loading || Boolean(dialog.error)}
        onClick={onSubmit}
      >
        {t('common.save')}
      </Button>
    </DialogActions>
  );
}

type FormContentProps = Readonly<{
  form: NoticeInput;
  setForm: React.Dispatch<React.SetStateAction<NoticeInput>>;
  tab: 'edit' | 'preview';
  setTab: (tab: 'edit' | 'preview') => void;
  attempted: boolean;
  loading: boolean;
  error?: unknown;
}>;

function NoticeFormContent(props: FormContentProps) {
  const { t } = useTranslate('admin');
  if (props.loading) {
    return (
      <Box sx={{ py: 10, display: 'flex', justifyContent: 'center' }}>
        <CircularProgress />
      </Box>
    );
  }
  if (props.error) return <Alert severity="error">{getErrorMessage(props.error)}</Alert>;
  return (
    <Stack sx={{ pt: 1, gap: 2.5 }}>
      <NoticeMetadataFields form={props.form} setForm={props.setForm} attempted={props.attempted} />
      <Tabs value={props.tab} onChange={(_, value: 'edit' | 'preview') => props.setTab(value)}>
        <Tab value="edit" label={t('notice.editor.edit')} />
        <Tab value="preview" label={t('notice.editor.preview')} />
      </Tabs>
      <NoticeContentEditor
        form={props.form}
        setForm={props.setForm}
        preview={props.tab === 'preview'}
      />
    </Stack>
  );
}

function NoticeMetadataFields(props: Pick<FormContentProps, 'form' | 'setForm' | 'attempted'>) {
  const { t } = useTranslate('admin');
  const error = props.attempted ? noticeTitleError(props.form) : null;
  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2}>
      <TextField
        required
        fullWidth
        label={t('notice.fields.title')}
        value={props.form.notice_title}
        error={Boolean(error)}
        helperText={error ? t(`notice.validation.${error}`, { max: NOTICE_TITLE_MAX_LENGTH }) : ' '}
        onChange={(event) => updateForm(props.setForm, 'notice_title', event.target.value)}
      />
      <NoticeSelectFields form={props.form} setForm={props.setForm} />
    </Stack>
  );
}

function NoticeSelectFields(props: Pick<FormContentProps, 'form' | 'setForm'>) {
  const { t } = useTranslate('admin');
  return (
    <>
      <TextField
        select
        required
        label={t('notice.fields.type')}
        value={props.form.notice_type}
        sx={{ minWidth: 150 }}
        onChange={(event) =>
          updateForm(props.setForm, 'notice_type', event.target.value as NoticeType)
        }
      >
        {Object.values(NOTICE_TYPE).map((type) => (
          <MenuItem key={type} value={type}>
            {t(noticeTypeTranslationKeys[type])}
          </MenuItem>
        ))}
      </TextField>
      <TextField
        select
        label={t('common.status')}
        value={props.form.status}
        sx={{ minWidth: 130 }}
        onChange={(event) =>
          updateForm(props.setForm, 'status', event.target.value as NoticeStatus)
        }
      >
        {Object.values(NOTICE_STATUS).map((status) => (
          <MenuItem key={status} value={status}>
            {t(noticeStatusTranslationKeys[status])}
          </MenuItem>
        ))}
      </TextField>
    </>
  );
}

function NoticeContentEditor(
  props: Pick<FormContentProps, 'form' | 'setForm'> & { preview: boolean }
) {
  const { t } = useTranslate('admin');
  if (props.preview) {
    return props.form.notice_content ? (
      <Box sx={{ minHeight: 300, p: 2, border: 1, borderColor: 'divider', borderRadius: 1 }}>
        <Markdown allowRawHtml={false} sourceFormat="markdown">
          {props.form.notice_content}
        </Markdown>
      </Box>
    ) : (
      <Alert severity="info">{t('notice.editor.emptyPreview')}</Alert>
    );
  }
  return (
    <>
      <TextField
        multiline
        fullWidth
        minRows={14}
        label={t('notice.fields.content')}
        value={props.form.notice_content}
        onChange={(event) => updateForm(props.setForm, 'notice_content', event.target.value)}
      />
      <TextField
        multiline
        fullWidth
        minRows={2}
        label={t('common.remark')}
        value={props.form.remark ?? ''}
        onChange={(event) => updateForm(props.setForm, 'remark', event.target.value)}
      />
    </>
  );
}

function updateForm<Key extends keyof NoticeInput>(
  setForm: React.Dispatch<React.SetStateAction<NoticeInput>>,
  key: Key,
  value: NoticeInput[Key]
) {
  setForm((current) => ({ ...current, [key]: value }));
}

'use client';

import type { Dispatch, SetStateAction } from 'react';
import type { CronBuilderState } from '../../model/cron/cron-builder-model';
import type { CronEditorModel } from '../../model/cron/cron-builder-parser';

import { useMemo, useState, useEffect, useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import Paper from '@mui/material/Paper';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TextField from '@mui/material/TextField';
import DialogTitle from '@mui/material/DialogTitle';
import ToggleButton from '@mui/material/ToggleButton';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import ToggleButtonGroup from '@mui/material/ToggleButtonGroup';

import { useTranslate } from 'src/shared/i18n/use-locales';

import { cronNextTimes } from '../../api';
import { CronResultPanel } from './cron-result-panel';
import { CronFieldPanels } from './cron-field-panels';
import { schedulerMutationErrorMessage } from '../../model/mutation-error';
import { CRON_FIELDS, defaultCronState } from '../../model/cron/cron-builder-model';
import {
  cronExpressionParts,
  changeCronEditorMode,
  cronEditorExpression,
  createCronEditorModel,
} from '../../model/cron/cron-builder-parser';

const CRON_PREVIEW_DEBOUNCE_MS = 250;

type Translate = (key: string, options?: Record<string, unknown>) => string;

type CronDialogProps = {
  open: boolean;
  expression: string;
  canPreview: boolean;
  onClose: () => void;
  onApply: (value: string) => void;
};

export function CronDialog(props: CronDialogProps) {
  const { t } = useTranslate('scheduler');
  const currentYear = useMemo(() => new Date().getFullYear(), []);
  const [editor, setEditor] = useCronEditor(props, currentYear);
  const value = cronEditorExpression(editor);
  const preview = useCronPreview({ open: props.open, enabled: props.canPreview, value, t });
  return (
    <Dialog fullWidth maxWidth="lg" open={props.open} onClose={props.onClose}>
      <DialogTitle>{t('cronBuilder')}</DialogTitle>
      <DialogContent>
        <CronEditorBody
          editor={editor}
          currentYear={currentYear}
          preview={preview}
          setEditor={setEditor}
        />
      </DialogContent>
      <DialogActions>
        <Button variant="contained" disabled={!preview.valid} onClick={() => props.onApply(value)}>
          {t('cronBuilderUi.confirm')}
        </Button>
        <Button
          color="warning"
          variant="outlined"
          onClick={() => setEditor(resetEditor(editor, currentYear))}
        >
          {t('admin:common.reset')}
        </Button>
        <Button variant="outlined" onClick={props.onClose}>
          {t('admin:common.cancel')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

type CronEditorBodyProps = {
  editor: CronEditorModel;
  currentYear: number;
  preview: CronPreview;
  setEditor: Dispatch<SetStateAction<CronEditorModel>>;
};

function CronEditorBody(props: CronEditorBodyProps) {
  const { t } = useTranslate('scheduler');
  const value = cronEditorExpression(props.editor);
  return (
    <Stack spacing={2.5} sx={{ pt: 1 }}>
      <CronModeControl
        editor={props.editor}
        currentYear={props.currentYear}
        setEditor={props.setEditor}
      />
      {props.editor.mode === 'builder' ? (
        <CronBuilderEditor
          editor={props.editor}
          currentYear={props.currentYear}
          setEditor={props.setEditor}
        />
      ) : (
        <TextField
          fullWidth
          label={t('customExpression')}
          value={props.editor.value}
          onChange={(event) =>
            props.setEditor(updateCustomEditor(props.editor, event.target.value))
          }
        />
      )}
      <CronResultPanel
        error={props.preview.error}
        expression={value}
        loading={props.preview.loading}
        parts={cronExpressionParts(value)}
        times={props.preview.times}
        t={t}
      />
    </Stack>
  );
}

function CronModeControl(props: Pick<CronEditorBodyProps, 'editor' | 'currentYear' | 'setEditor'>) {
  const { t } = useTranslate('scheduler');
  return (
    <ToggleButtonGroup
      exclusive
      size="small"
      value={props.editor.mode}
      onChange={(_, mode: CronEditorModel['mode'] | null) => {
        if (mode) props.setEditor(changeCronEditorMode(props.editor, mode, props.currentYear));
      }}
    >
      <ToggleButton value="builder">{t('mode.builder')}</ToggleButton>
      <ToggleButton value="custom">{t('mode.custom')}</ToggleButton>
    </ToggleButtonGroup>
  );
}

function CronBuilderEditor(
  props: Pick<CronEditorBodyProps, 'editor' | 'currentYear' | 'setEditor'>
) {
  const { t } = useTranslate('scheduler');
  const [tabIndex, setTabIndex] = useState(0);
  const setState = useBuilderStateSetter(props.editor, props.setEditor);
  if (props.editor.mode !== 'builder') return null;
  return (
    <Paper variant="outlined">
      <Tabs value={tabIndex} variant="scrollable" onChange={(_, value) => setTabIndex(value)}>
        {CRON_FIELDS.map((field) => (
          <Tab key={field} label={t(`cron.${field}`)} />
        ))}
      </Tabs>
      <Stack sx={{ p: 2 }}>
        <CronFieldPanels
          activeField={CRON_FIELDS[tabIndex]}
          currentYear={props.currentYear}
          state={props.editor.state}
          t={t}
          setState={setState}
        />
      </Stack>
    </Paper>
  );
}

function useCronEditor(props: CronDialogProps, currentYear: number) {
  const [editor, setEditor] = useState(() => createCronEditorModel(props.expression, currentYear));
  useEffect(() => {
    if (props.open) setEditor(createCronEditorModel(props.expression, currentYear));
  }, [currentYear, props.expression, props.open]);
  return [editor, setEditor] as const;
}

function useBuilderStateSetter(
  editor: CronEditorModel,
  setEditor: Dispatch<SetStateAction<CronEditorModel>>
) {
  return useCallback<Dispatch<SetStateAction<CronBuilderState>>>(
    (action) => {
      if (editor.mode !== 'builder') return;
      const state = typeof action === 'function' ? action(editor.state) : action;
      setEditor({ ...editor, state, dirty: true });
    },
    [editor, setEditor]
  );
}

type CronPreview = { error: string; loading: boolean; times: string[]; valid: boolean };
type CronPreviewOptions = { open: boolean; enabled: boolean; value: string; t: Translate };

function useCronPreview(options: CronPreviewOptions): CronPreview {
  const [result, setResult] = useState({
    error: '',
    loading: false,
    times: [] as string[],
    validated: '',
  });
  useEffect(() => {
    if (!options.open || !options.enabled) return undefined;
    let active = true;
    setResult({ error: '', loading: true, times: [], validated: '' });
    const timer = window.setTimeout(() => {
      loadPreview(options.value, options.t).then((next) => {
        if (active) setResult(next);
      });
    }, CRON_PREVIEW_DEBOUNCE_MS);
    return () => {
      active = false;
      window.clearTimeout(timer);
    };
  }, [options.enabled, options.open, options.t, options.value]);
  const valid = !result.loading && !result.error && result.validated === options.value;
  return { error: result.error, loading: result.loading, times: result.times, valid };
}

async function loadPreview(value: string, t: Translate) {
  try {
    const result = await cronNextTimes(value);
    return { error: '', loading: false, times: result.times, validated: value };
  } catch (error) {
    const message = schedulerMutationErrorMessage(error, t('cronInvalid'));
    return { error: message, loading: false, times: [], validated: '' };
  }
}

function updateCustomEditor(editor: CronEditorModel, value: string): CronEditorModel {
  if (editor.mode !== 'custom') return editor;
  return { ...editor, value, dirty: value !== editor.original };
}

function resetEditor(editor: CronEditorModel, currentYear: number): CronEditorModel {
  if (editor.mode === 'custom') return { ...editor, value: editor.original, dirty: false };
  const state = defaultCronState(currentYear);
  return {
    ...editor,
    state,
    dirty: cronEditorExpression({ ...editor, state, dirty: true }) !== editor.original,
  };
}

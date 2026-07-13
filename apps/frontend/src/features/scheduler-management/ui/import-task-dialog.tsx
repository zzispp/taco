'use client';

import type { ImportableTask } from 'src/entities/scheduler';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

type ImportTaskDialogProps = {
  open: boolean;
  tasks: ImportableTask[];
  loading: boolean;
  error?: unknown;
  onClose: () => void;
  onSelect: (task: ImportableTask) => void;
};

export function ImportTaskDialog(props: ImportTaskDialogProps) {
  const { t } = useTranslate('scheduler');
  const { t: tAdmin } = useTranslate('admin');
  return (
    <Dialog fullWidth maxWidth="md" open={props.open} onClose={props.onClose}>
      <DialogTitle>{t('importJob')}</DialogTitle>
      <DialogContent>
        <ImportTaskContent props={props} />
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={props.onClose}>
          {tAdmin('common.cancel')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function ImportTaskContent({ props }: { props: ImportTaskDialogProps }) {
  const { t } = useTranslate('admin');
  if (props.error) return <Alert severity="error">{getErrorMessage(props.error)}</Alert>;
  return (
    <Stack spacing={1.5} sx={{ pt: 1 }}>
      {props.loading && <Typography>{t('common.loading')}</Typography>}
      {props.tasks.map((task) => (
        <Button key={task.task_key} variant="outlined" onClick={() => props.onSelect(task)}>
          {task.name} · {task.group_label} · {task.description}
        </Button>
      ))}
      {!props.loading && props.tasks.length === 0 && <Typography>{t('common.noData')}</Typography>}
    </Stack>
  );
}

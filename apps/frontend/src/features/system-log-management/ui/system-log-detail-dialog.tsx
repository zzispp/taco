import type { SystemLogController } from '../model/use-system-log-controller';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Typography from '@mui/material/Typography';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import CircularProgress from '@mui/material/CircularProgress';

import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { apiMutationErrorMessage } from 'src/shared/api/mutation-error';

export function SystemLogDetailDialog({ controller }: { controller: SystemLogController }) {
  const { t } = useTranslate('systemLog');
  const { state, actions } = controller;
  return (
    <Dialog
      fullWidth
      maxWidth="lg"
      open={Boolean(state.detailTarget)}
      onClose={actions.closeDetail}
    >
      <DialogTitle>{t('detail.title')}</DialogTitle>
      <DialogContent dividers>
        <DetailContent controller={controller} />
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={actions.closeDetail}>
          {t('cancel')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function DetailContent({ controller }: { controller: SystemLogController }) {
  const { t } = useTranslate('systemLog');
  const { detail } = controller.resources;
  if (detail.isLoading) return <CircularProgress size={28} />;
  if (detail.error)
    return (
      <Alert severity="error">
        {apiMutationErrorMessage(detail.error, t('messages.detailFailure'))}
      </Alert>
    );
  if (!detail.data) return <Typography color="text.secondary">{t('detail.empty')}</Typography>;
  const value = detail.data;
  return (
    <Stack spacing={3}>
      <Box
        sx={{
          display: 'grid',
          gridTemplateColumns: { xs: '1fr', md: 'repeat(2, minmax(0, 1fr))' },
          gap: 2,
        }}
      >
        <DetailItem label={t('fields.id')} value={value.log_id} />
        <DetailItem label={t('fields.level')} value={t(`levels.${value.level}`)} />
        <DetailItem label={t('fields.target')} value={value.target} />
        <DetailItem label={t('fields.occurredAt')} value={fAdminDateTime(value.occurred_at)} />
        <DetailItem label={t('fields.message')} value={value.message} />
      </Box>
      <Box component="section">
        <Typography variant="subtitle2" sx={{ mb: 1 }}>
          {t('fields.fields')}
        </Typography>
        <Box
          component="pre"
          sx={{
            m: 0,
            p: 2,
            maxHeight: 360,
            overflow: 'auto',
            bgcolor: 'background.neutral',
            borderRadius: 1,
            fontSize: 13,
          }}
        >
          {JSON.stringify(value.fields, null, 2)}
        </Box>
      </Box>
    </Stack>
  );
}

function DetailItem({ label, value }: { label: string; value: string }) {
  return (
    <Box>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Typography sx={{ overflowWrap: 'anywhere' }}>{value || '-'}</Typography>
    </Box>
  );
}

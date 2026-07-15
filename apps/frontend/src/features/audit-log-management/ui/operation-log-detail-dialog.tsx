import type { OperationLogController } from '../model/use-operation-log-controller';

import Box from '@mui/material/Box';
import Alert from '@mui/material/Alert';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import Tooltip from '@mui/material/Tooltip';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';
import CircularProgress from '@mui/material/CircularProgress';

import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { getErrorMessage } from 'src/shared/lib/get-error-message';

import {
  auditStatusKeys,
  operationBusinessTypeKeys,
  operationOperatorTypeKeys,
} from 'src/entities/audit-log';

import { auditDetailSections } from '../model/detail-format';

export function OperationLogDetailDialog({ controller }: { controller: OperationLogController }) {
  const { t } = useTranslate('audit');
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

function DetailContent({ controller }: { controller: OperationLogController }) {
  const { t } = useTranslate('audit');
  const { resources } = controller;
  if (resources.detail.isLoading) {
    return <CircularProgress size={28} />;
  }
  if (resources.detail.error) {
    return <Alert severity="error">{getErrorMessage(resources.detail.error)}</Alert>;
  }
  const detail = resources.detail.data;
  if (!detail) return <Typography color="text.secondary">{t('detail.empty')}</Typography>;
  return (
    <Stack spacing={3}>
      <DetailMetadata detail={detail} />
      {auditDetailSections(detail).map((section) => (
        <DetailSection key={section.key} label={t(`detail.${section.key}`)} value={section.value} />
      ))}
    </Stack>
  );
}

function DetailMetadata({
  detail,
}: {
  detail: NonNullable<OperationLogController['resources']['detail']['data']>;
}) {
  const { t } = useTranslate('audit');
  const items = [
    [t('fields.title'), detail.title],
    [t('fields.businessType'), t(operationBusinessTypeKeys[detail.business_type])],
    [t('fields.operatorType'), t(operationOperatorTypeKeys[detail.operator_type])],
    [t('fields.operator'), detail.oper_name ?? ''],
    [t('fields.department'), detail.dept_name ?? ''],
    [t('fields.requestMethod'), detail.request_method],
    [t('fields.method'), detail.method],
    [t('fields.url'), detail.oper_url],
    [t('fields.operationIp'), detail.oper_ip],
    [t('fields.operationLocation'), detail.oper_location],
    [t('fields.operationStatus'), t(auditStatusKeys[detail.status])],
    [t('fields.operationTime'), fAdminDateTime(detail.oper_time)],
    [t('fields.duration'), t('table.milliseconds', { value: detail.cost_time })],
  ];
  return (
    <Box
      sx={{
        display: 'grid',
        gridTemplateColumns: { xs: '1fr', md: 'repeat(2, minmax(0, 1fr))' },
        gap: 2,
      }}
    >
      {items.map(([label, value]) => (
        <Box key={label} sx={{ minWidth: 0 }}>
          <Typography variant="caption" color="text.secondary">
            {label}
          </Typography>
          <Typography sx={{ overflowWrap: 'anywhere' }}>{value || '-'}</Typography>
        </Box>
      ))}
    </Box>
  );
}

function DetailSection({ label, value }: { label: string; value: string }) {
  const { t } = useTranslate('audit');
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      toast.success(t('detail.copySuccess'));
    } catch {
      toast.error(t('detail.copyFailure'));
    }
  };
  return (
    <Box component="section">
      <Stack direction="row" alignItems="center" justifyContent="space-between" sx={{ mb: 1 }}>
        <Typography variant="subtitle2">{label}</Typography>
        <Tooltip title={t('actions.copy')}>
          <span>
            <IconButton
              size="small"
              disabled={!value}
              aria-label={t('actions.copy')}
              onClick={copy}
            >
              <Iconify icon="solar:copy-bold" />
            </IconButton>
          </span>
        </Tooltip>
      </Stack>
      <Box
        component="pre"
        sx={{
          m: 0,
          p: 1.5,
          minHeight: 56,
          maxHeight: 280,
          overflow: 'auto',
          border: (theme) => `1px solid ${theme.palette.divider}`,
          borderRadius: 1,
          bgcolor: 'background.neutral',
          fontFamily: 'monospace',
          fontSize: 13,
          whiteSpace: 'pre-wrap',
          overflowWrap: 'anywhere',
        }}
      >
        {value || t('detail.empty')}
      </Box>
    </Box>
  );
}

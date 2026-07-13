import type { CronParts } from '../../model/cron/cron-builder-model';

import Alert from '@mui/material/Alert';
import Paper from '@mui/material/Paper';
import Table from '@mui/material/Table';
import Stack from '@mui/material/Stack';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import TableHead from '@mui/material/TableHead';
import Typography from '@mui/material/Typography';
import TableContainer from '@mui/material/TableContainer';

import { CRON_FIELDS } from '../../model/cron/cron-builder-model';

type Translate = (key: string, options?: Record<string, unknown>) => string;

type CronResultPanelProps = {
  error: string;
  expression: string;
  loading: boolean;
  parts: CronParts;
  times: string[];
  t: Translate;
};

export function CronResultPanel({
  error,
  expression,
  loading,
  parts,
  times,
  t,
}: CronResultPanelProps) {
  return (
    <Stack spacing={2}>
      <Paper variant="outlined" sx={{ p: 2 }}>
        <Typography variant="subtitle2" sx={{ mb: 1 }}>
          {t('cronBuilderUi.timeExpression')}
        </Typography>
        <TableContainer>
          <Table size="small">
            <TableHead>
              <TableRow>
                {CRON_FIELDS.map((field) => (
                  <TableCell key={field}>{t(`cron.${field}`)}</TableCell>
                ))}
                <TableCell>{t('cronExpression')}</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              <TableRow>
                {CRON_FIELDS.map((field) => (
                  <TableCell key={field}>{parts[field]}</TableCell>
                ))}
                <TableCell>{expression}</TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </TableContainer>
      </Paper>
      <Paper variant="outlined" sx={{ p: 2 }}>
        <Typography variant="subtitle2" sx={{ mb: 1 }}>
          {t('nextTimes')}
        </Typography>
        <NextTimesList error={error} loading={loading} times={times} t={t} />
      </Paper>
    </Stack>
  );
}

function NextTimesList({
  error,
  loading,
  times,
  t,
}: Pick<CronResultPanelProps, 'error' | 'loading' | 'times' | 't'>) {
  if (error) return <Alert severity="error">{error}</Alert>;
  if (loading)
    return <Typography color="text.secondary">{t('cronBuilderUi.calculating')}</Typography>;
  if (!times.length) return <Typography color="text.secondary">{t('noPreview')}</Typography>;
  return (
    <Stack component="ul" spacing={0.5} sx={{ m: 0, pl: 2.5 }}>
      {times.map((time) => (
        <Typography component="li" key={time} variant="body2">
          {time}
        </Typography>
      ))}
    </Stack>
  );
}

'use client';

import type { TableHeadCellProps } from 'src/shared/ui/table';
import type { NoticeManagementController } from 'src/features/notice-management';

import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Dialog from '@mui/material/Dialog';
import TableRow from '@mui/material/TableRow';
import TextField from '@mui/material/TextField';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import DialogTitle from '@mui/material/DialogTitle';
import DialogActions from '@mui/material/DialogActions';
import DialogContent from '@mui/material/DialogContent';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { getErrorMessage } from 'src/shared/lib/get-error-message';
import { TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import { TableLoadingRows, ManagementTableHead } from 'src/shared/ui/admin';

export function NoticeReaderDialog({ controller }: { controller: NoticeManagementController }) {
  const { t } = useTranslate('admin');
  const { state } = controller;
  const target = state.readerTarget;
  return (
    <Dialog
      fullWidth
      maxWidth="lg"
      open={Boolean(target)}
      onClose={() => state.setReaderTarget(null)}
    >
      <DialogTitle>{t('notice.readersTitle', { name: target?.notice_title ?? '' })}</DialogTitle>
      <DialogContent>
        <ReaderSearch controller={controller} />
        <ReaderTable controller={controller} />
      </DialogContent>
      <DialogActions>
        <Button variant="outlined" onClick={() => state.setReaderTarget(null)}>
          {t('common.close')}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

function ReaderSearch({ controller }: { controller: NoticeManagementController }) {
  const { t } = useTranslate('admin');
  const { state, actions } = controller;
  return (
    <Stack direction={{ xs: 'column', sm: 'row' }} spacing={2} sx={{ mb: 2, pt: 1 }}>
      <TextField
        fullWidth
        size="small"
        label={t('notice.readerSearch')}
        value={state.readerDraft}
        onChange={(event) => state.setReaderDraft(event.target.value)}
        onKeyDown={(event) => {
          if (event.key === 'Enter') actions.searchReaders();
        }}
      />
      <Button variant="outlined" onClick={actions.searchReaders}>
        {t('common.search')}
      </Button>
    </Stack>
  );
}

function ReaderTable({ controller }: { controller: NoticeManagementController }) {
  const { t } = useTranslate('admin');
  const { state, resources } = controller;
  const head = readerHead(t);
  if (resources.readers.error) {
    return <Alert severity="error">{getErrorMessage(resources.readers.error)}</Alert>;
  }
  return (
    <>
      <Scrollbar>
        <Table size="small" sx={{ minWidth: 850 }}>
          <ManagementTableHead head={head} rowCount={resources.readers.items.length} />
          <TableBody>
            {resources.readers.isLoading ? (
              <TableLoadingRows head={head} rows={state.readerRowsPerPage} />
            ) : (
              resources.readers.items.map((reader) => (
                <TableRow hover key={`${reader.user_id}-${reader.read_time}`}>
                  <TableCell>{reader.user_name}</TableCell>
                  <TableCell>{reader.nick_name}</TableCell>
                  <TableCell>{reader.dept_name ?? '-'}</TableCell>
                  <TableCell>{reader.phonenumber ?? '-'}</TableCell>
                  <TableCell>{fAdminDateTime(reader.read_time)}</TableCell>
                </TableRow>
              ))
            )}
            <TableNoData
              title={t('common.noData')}
              notFound={!resources.readers.isLoading && resources.readers.items.length === 0}
            />
          </TableBody>
        </Table>
      </Scrollbar>
      <TablePaginationCustom
        page={state.readerPage}
        count={resources.readers.total}
        rowsPerPage={state.readerRowsPerPage}
        onPageChange={(_, page) => state.setReaderPage(page)}
        onRowsPerPageChange={(event) => {
          state.setReaderRowsPerPage(Number(event.target.value));
          state.setReaderPage(0);
        }}
      />
    </>
  );
}

function readerHead(t: ReturnType<typeof useTranslate>['t']): TableHeadCellProps[] {
  return [
    { id: 'user_name', label: t('common.username') },
    { id: 'nick_name', label: t('fields.nickName') },
    { id: 'dept_name', label: t('fields.deptName') },
    { id: 'phonenumber', label: t('fields.phone') },
    { id: 'read_time', label: t('notice.readTime') },
  ];
}

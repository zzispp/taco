import type { ServerDashboard } from 'src/entities/system';

import Card from '@mui/material/Card';
import Table from '@mui/material/Table';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import CardHeader from '@mui/material/CardHeader';
import Typography from '@mui/material/Typography';
import TableContainer from '@mui/material/TableContainer';

import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { formatBytes, formatPercent } from './format';

export function TopProcessesCard({ dashboard }: { dashboard: ServerDashboard }) {
  const { t } = useTranslate('admin');
  return (
    <Card>
      <CardHeader
        title={t('systemDashboard.tables.topProcesses')}
        subheader={t('systemDashboard.tables.sortedByCpu')}
      />
      <Scrollbar>
        <TableContainer sx={{ minWidth: 720 }}>
          <Table>
            <TableBody>
              {dashboard.top_processes.map((item) => (
                <ProcessRow key={item.pid} item={item} />
              ))}
            </TableBody>
          </Table>
        </TableContainer>
      </Scrollbar>
    </Card>
  );
}

export function CpuCoresCard({ dashboard }: { dashboard: ServerDashboard }) {
  const { t } = useTranslate('admin');
  return (
    <Card>
      <CardHeader
        title={t('systemDashboard.tables.cpuCoreLoad')}
        subheader={t('systemDashboard.tables.cpuCoreLoadSubheader')}
      />
      <Table>
        <TableBody>
          {dashboard.cpu.cores.map((item) => (
            <CpuCoreRow key={item.name} item={item} />
          ))}
        </TableBody>
      </Table>
    </Card>
  );
}

export function DiskPartitionsCard({ dashboard }: { dashboard: ServerDashboard }) {
  const { t } = useTranslate('admin');
  return (
    <Card>
      <CardHeader
        title={t('systemDashboard.tables.diskPartitions')}
        subheader={t('systemDashboard.tables.diskPartitionsSubheader')}
      />
      <Table>
        <TableBody>
          {dashboard.disks.map((item) => (
            <DiskRow key={item.mount_point} item={item} />
          ))}
        </TableBody>
      </Table>
    </Card>
  );
}

function ProcessRow({ item }: { item: ServerDashboard['top_processes'][number] }) {
  return (
    <TableRow>
      <TableCell>{item.name}</TableCell>
      <TableCell>{item.pid}</TableCell>
      <TableCell>{formatPercent(item.cpu_usage_percent)}</TableCell>
      <TableCell>{formatBytes(item.memory_bytes)}</TableCell>
      <TableCell>
        {formatBytes(item.disk_read_bytes)} / {formatBytes(item.disk_written_bytes)}
      </TableCell>
    </TableRow>
  );
}

function CpuCoreRow({ item }: { item: ServerDashboard['cpu']['cores'][number] }) {
  return (
    <TableRow>
      <TableCell>{item.name}</TableCell>
      <TableCell>{formatPercent(item.usage_percent)}</TableCell>
      <TableCell>{item.frequency_mhz} MHz</TableCell>
    </TableRow>
  );
}

function DiskRow({ item }: { item: ServerDashboard['disks'][number] }) {
  return (
    <TableRow>
      <TableCell>
        <Typography variant="body2">{item.mount_point}</Typography>
        <Typography variant="caption" color="text.secondary">
          {item.file_system}
        </Typography>
      </TableCell>
      <TableCell>{formatBytes(item.used_bytes)}</TableCell>
      <TableCell>{formatBytes(item.available_bytes)}</TableCell>
      <TableCell>{formatPercent(item.usage_percent)}</TableCell>
    </TableRow>
  );
}

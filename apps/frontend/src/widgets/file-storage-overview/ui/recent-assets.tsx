import type { FileOverview } from 'src/entities/file';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';

import { RouterLink } from 'src/shared/routes/components';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { ManagedFileThumbnail } from 'src/entities/file';

import { buildFileManagerEntryPath } from 'src/features/file-management';

const RECENT_ASSET_LIMIT = 8;

export function RecentAssets({ overview }: { overview: FileOverview }) {
  const { t } = useTranslate('admin');
  const entries = [...overview.recent_folders, ...overview.recent_entries]
    .sort((left, right) => Date.parse(right.updated_at) - Date.parse(left.updated_at))
    .slice(0, RECENT_ASSET_LIMIT);
  return (
    <Card variant="outlined" sx={{ p: 2.5 }}>
      <Stack spacing={2}>
        <Typography variant="subtitle1">{t('file.recentAssets')}</Typography>
        {entries.length ? (
          entries.map((entry) => (
            <Box
              key={entry.id}
              component={RouterLink}
              href={buildFileManagerEntryPath(entry)}
              sx={{
                display: 'flex',
                alignItems: 'center',
                gap: 1.5,
                color: 'inherit',
                textDecoration: 'none',
              }}
            >
              <ManagedFileThumbnail
                entry={entry}
                errorLabel={t('file.messages.thumbnailFailed')}
                sx={{ width: 36, height: 36 }}
              />
              <Box sx={{ minWidth: 0, flex: 1 }}>
                <Typography noWrap variant="body2">
                  {entry.name}
                </Typography>
                <Typography noWrap variant="caption" color="text.secondary">
                  {fAdminDateTime(entry.updated_at)}
                </Typography>
              </Box>
            </Box>
          ))
        ) : (
          <Typography color="text.secondary">{t('file.empty')}</Typography>
        )}
      </Stack>
    </Card>
  );
}

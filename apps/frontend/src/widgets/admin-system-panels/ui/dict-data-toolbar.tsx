import type { TranslateFn } from 'src/shared/i18n';

import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import Typography from '@mui/material/Typography';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';

export function DictDataToolbar(props: DictDataToolbarProps) {
  const { t } = useTranslate('admin');
  return (
    <Stack direction="row" justifyContent="space-between" alignItems="center" sx={{ p: 2 }}>
      <Typography variant="h6">
        {t('fields.dictData')}：{props.activeType || '-'}
      </Typography>
      <DictDataActions props={props} t={t} />
    </Stack>
  );
}

function DictDataActions({ props, t }: { props: DictDataToolbarProps; t: TranslateFn }) {
  return (
    <Stack direction="row" spacing={1}>
      {props.canExport && (
        <Button
          variant="outlined"
          startIcon={<Iconify icon="solar:export-bold" />}
          disabled={!props.activeType}
          onClick={props.onExport}
        >
          {t('actions.export')}
        </Button>
      )}
      {props.canRemove && (
        <Button
          variant="outlined"
          color="error"
          disabled={props.selectedCount === 0}
          onClick={props.onBatchDelete}
        >
          {t('common.delete')}
        </Button>
      )}
      {props.canAdd && (
        <Button
          variant="contained"
          startIcon={<Iconify icon="mingcute:add-line" />}
          disabled={!props.activeType}
          onClick={props.onAdd}
        >
          {t('actions.addDictData')}
        </Button>
      )}
    </Stack>
  );
}

type DictDataToolbarProps = Readonly<{
  activeType: string;
  selectedCount: number;
  canAdd: boolean;
  canRemove: boolean;
  canExport: boolean;
  onAdd: () => void;
  onBatchDelete: () => void;
  onExport: () => void;
}>;

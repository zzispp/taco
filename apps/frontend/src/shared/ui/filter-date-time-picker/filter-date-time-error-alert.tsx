import type { LocalDateTimeFilterError } from 'src/shared/lib/local-date-time-filter';

import Alert from '@mui/material/Alert';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY } from 'src/shared/lib/local-date-time-filter';

export function FilterDateTimeErrorAlert({ error }: FilterDateTimeErrorAlertProps) {
  const { t } = useTranslate('admin');
  if (!error) return null;

  return (
    <Alert severity="error" role="alert" sx={{ mt: 2 }}>
      {t(LOCAL_DATE_TIME_FILTER_ERROR_TRANSLATION_KEY[error])}
    </Alert>
  );
}

type FilterDateTimeErrorAlertProps = Readonly<{
  error: LocalDateTimeFilterError | null;
}>;

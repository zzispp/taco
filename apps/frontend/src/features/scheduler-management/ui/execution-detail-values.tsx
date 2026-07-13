import type {
  CapturedBytes,
  CapturedHeader,
  HttpExecutionFailure,
  HttpExecutionRequest,
  HttpExecutionResponse,
} from 'src/entities/scheduler';

import Box from '@mui/material/Box';
import Chip from '@mui/material/Chip';
import Stack from '@mui/material/Stack';
import Alert from '@mui/material/Alert';
import Divider from '@mui/material/Divider';
import Typography from '@mui/material/Typography';

import { useTranslate } from 'src/shared/i18n/use-locales';

import {
  CAPTURED_BYTES_ENCODING,
  httpExecutionFailureTranslationKeys,
} from 'src/entities/scheduler';

import { formatRawJson } from '../model/execution-detail';

const RAW_CONTENT_MAX_HEIGHT = 360;

export function HttpRequestDetail({ request }: { request: HttpExecutionRequest }) {
  const { t } = useTranslate('scheduler');
  return (
    <Stack spacing={3}>
      <Metadata label={t('executionDetail.fields.method')} value={request.method} />
      <Metadata label={t('executionDetail.fields.url')} value={request.url} />
      <CapturedHeaders headers={request.headers} />
      <CapturedBody body={request.body} />
    </Stack>
  );
}

export function HttpResponseDetail(props: {
  failure: HttpExecutionFailure | null;
  response: HttpExecutionResponse | null;
}) {
  const { t } = useTranslate('scheduler');
  if (!props.response) {
    return (
      <Stack spacing={2}>
        <Alert severity="info">{t('executionDetail.noResponse')}</Alert>
        {props.failure && (
          <Alert severity="error">
            {t(httpExecutionFailureTranslationKeys[props.failure.code])}
          </Alert>
        )}
      </Stack>
    );
  }
  return (
    <Stack spacing={3}>
      {props.failure && (
        <Alert severity="error">{t(httpExecutionFailureTranslationKeys[props.failure.code])}</Alert>
      )}
      <Metadata label={t('executionDetail.fields.statusCode')} value={props.response.status} />
      <Metadata label={t('executionDetail.fields.finalUrl')} value={props.response.final_url} />
      <CapturedHeaders headers={props.response.headers} />
      <CapturedBody body={props.response.body} />
    </Stack>
  );
}

export function RawDetail({ value }: { value: unknown }) {
  const content = typeof value === 'string' ? value : formatRawJson(value);
  return <RawContent content={content} />;
}

function CapturedHeaders({ headers }: { headers: readonly CapturedHeader[] }) {
  const { t } = useTranslate('scheduler');
  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{t('executionDetail.fields.headers')}</Typography>
      {headers.length === 0 ? (
        <Typography color="text.secondary" variant="body2">
          {t('executionDetail.noHeaders')}
        </Typography>
      ) : (
        headers.map((header, index) => (
          <Box key={`${header.name}-${index}`}>
            <Typography variant="caption" sx={{ display: 'block', mb: 0.5 }}>
              {header.name}
            </Typography>
            <CapturedValue value={header.value} />
          </Box>
        ))
      )}
    </Stack>
  );
}

function CapturedBody({ body }: { body: CapturedBytes | null }) {
  const { t } = useTranslate('scheduler');
  return (
    <Stack spacing={1.5}>
      <Typography variant="subtitle2">{t('executionDetail.fields.body')}</Typography>
      {body ? (
        <CapturedValue value={body} />
      ) : (
        <Typography color="text.secondary" variant="body2">
          {t('executionDetail.noBody')}
        </Typography>
      )}
    </Stack>
  );
}

function CapturedValue({ value }: { value: CapturedBytes }) {
  const { t } = useTranslate('scheduler');
  const encoding =
    value.encoding === CAPTURED_BYTES_ENCODING.UTF8
      ? t('executionDetail.encoding.utf8')
      : t('executionDetail.encoding.base64');
  return (
    <Stack spacing={1}>
      <Stack direction="row" spacing={1} useFlexGap flexWrap="wrap">
        <Chip size="small" label={encoding} />
        <Chip
          size="small"
          variant="outlined"
          label={t('executionDetail.byteLength', { value: value.byte_length })}
        />
      </Stack>
      <RawContent content={value.content} />
    </Stack>
  );
}

function RawContent({ content }: { content: string }) {
  return (
    <Box
      component="pre"
      sx={{
        m: 0,
        p: 2,
        maxHeight: RAW_CONTENT_MAX_HEIGHT,
        overflow: 'auto',
        borderRadius: 1,
        bgcolor: 'action.hover',
        fontSize: '0.8125rem',
        lineHeight: 1.6,
        whiteSpace: 'pre-wrap',
        overflowWrap: 'anywhere',
        fontFamily: 'monospace',
      }}
    >
      {content}
    </Box>
  );
}

function Metadata(props: { label: string; value: string | number }) {
  return (
    <Stack spacing={0.75}>
      <Typography variant="subtitle2">{props.label}</Typography>
      <Typography variant="body2" sx={{ overflowWrap: 'anywhere' }}>
        {props.value}
      </Typography>
      <Divider />
    </Stack>
  );
}

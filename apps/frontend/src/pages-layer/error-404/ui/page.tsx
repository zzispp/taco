'use client';

import { m } from 'framer-motion';

import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { paths } from 'src/shared/routes/paths';
import { RouterLink } from 'src/shared/routes/components';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { varBounce, MotionContainer } from 'src/shared/ui/animate';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { PageNotFoundIllustration } from 'src/shared/assets/illustrations';
import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SimpleLayout } from 'src/widgets/simple-shell';

// ----------------------------------------------------------------------

type NotFoundViewProps = Readonly<{
  homeHref?: string;
}>;

export function NotFoundView({ homeHref = paths.home }: NotFoundViewProps) {
  const { siteName } = useSiteDisplay();
  const { t } = useTranslate('common');

  return (
    <SimpleLayout
      homeHref={homeHref}
      slotProps={{
        content: { compact: true },
      }}
    >
      <SiteDocumentTitle title={formatErrorDocumentTitle(t('error404.title'), siteName)} />

      <Container component={MotionContainer}>
        <m.div variants={varBounce('in')}>
          <Typography variant="h3" sx={{ mb: 2 }}>
            {t('error404.title')}
          </Typography>
        </m.div>

        <m.div variants={varBounce('in')}>
          <Typography sx={{ color: 'text.secondary' }}>
            {t('error404.description')}
          </Typography>
        </m.div>

        <m.div variants={varBounce('in')}>
          <PageNotFoundIllustration sx={{ my: { xs: 5, sm: 10 } }} />
        </m.div>

        <Button component={RouterLink} href={homeHref} size="large" variant="contained">
          {t('error404.home')}
        </Button>
      </Container>
    </SimpleLayout>
  );
}

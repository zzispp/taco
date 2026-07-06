'use client';

import { m } from 'framer-motion';

import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { RouterLink } from 'src/shared/routes/components';
import { varBounce, MotionContainer } from 'src/shared/ui/animate';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { ServerErrorIllustration } from 'src/shared/assets/illustrations';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SimpleLayout } from 'src/widgets/simple-shell';

// ----------------------------------------------------------------------

export function View500() {
  const { siteName } = useSiteDisplay();

  return (
    <SimpleLayout
      slotProps={{
        content: { compact: true },
      }}
    >
      <SiteDocumentTitle title={formatErrorDocumentTitle('500 Internal server error!', siteName)} />

      <Container component={MotionContainer}>
        <m.div variants={varBounce('in')}>
          <Typography variant="h3" sx={{ mb: 2 }}>
            500 Internal server error
          </Typography>
        </m.div>

        <m.div variants={varBounce('in')}>
          <Typography sx={{ color: 'text.secondary' }}>
            There was an error, please try again later.
          </Typography>
        </m.div>

        <m.div variants={varBounce('in')}>
          <ServerErrorIllustration sx={{ my: { xs: 5, sm: 10 } }} />
        </m.div>

        <Button component={RouterLink} href="/" size="large" variant="contained">
          Go to home
        </Button>
      </Container>
    </SimpleLayout>
  );
}

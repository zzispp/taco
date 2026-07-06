'use client';

import { m } from 'framer-motion';

import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { RouterLink } from 'src/shared/routes/components';
import { varBounce, MotionContainer } from 'src/shared/ui/animate';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { PageNotFoundIllustration } from 'src/shared/assets/illustrations';
import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SimpleLayout } from 'src/widgets/simple-shell';

// ----------------------------------------------------------------------

export function NotFoundView() {
  const { siteName } = useSiteDisplay();

  return (
    <SimpleLayout
      slotProps={{
        content: { compact: true },
      }}
    >
      <SiteDocumentTitle title={formatErrorDocumentTitle('404 page not found!', siteName)} />

      <Container component={MotionContainer}>
        <m.div variants={varBounce('in')}>
          <Typography variant="h3" sx={{ mb: 2 }}>
            Sorry, page not found!
          </Typography>
        </m.div>

        <m.div variants={varBounce('in')}>
          <Typography sx={{ color: 'text.secondary' }}>
            Sorry, we couldn’t find the page you’re looking for. Perhaps you’ve mistyped the URL? Be
            sure to check your spelling.
          </Typography>
        </m.div>

        <m.div variants={varBounce('in')}>
          <PageNotFoundIllustration sx={{ my: { xs: 5, sm: 10 } }} />
        </m.div>

        <Button component={RouterLink} href="/" size="large" variant="contained">
          Go to home
        </Button>
      </Container>
    </SimpleLayout>
  );
}

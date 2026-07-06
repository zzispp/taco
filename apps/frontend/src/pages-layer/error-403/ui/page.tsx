'use client';

import { m } from 'framer-motion';

import Button from '@mui/material/Button';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';

import { RouterLink } from 'src/shared/routes/components';
import { varBounce, MotionContainer } from 'src/shared/ui/animate';
import { ForbiddenIllustration } from 'src/shared/assets/illustrations';
import { useSiteDisplay } from 'src/shared/config/site-display-context';
import { SiteDocumentTitle } from 'src/shared/config/site-document-title';
import { formatErrorDocumentTitle } from 'src/shared/i18n/document-title-format';

import { SimpleLayout } from 'src/widgets/simple-shell';

// ----------------------------------------------------------------------

export function View403() {
  const { siteName } = useSiteDisplay();

  return (
    <SimpleLayout
      slotProps={{
        content: { compact: true },
      }}
    >
      <SiteDocumentTitle title={formatErrorDocumentTitle('403 forbidden!', siteName)} />

      <Container component={MotionContainer}>
        <m.div variants={varBounce('in')}>
          <Typography variant="h3" sx={{ mb: 2 }}>
            No permission
          </Typography>
        </m.div>

        <m.div variants={varBounce('in')}>
          <Typography sx={{ color: 'text.secondary' }}>
            The page you’re trying to access has restricted access. Please refer to your system
            administrator.
          </Typography>
        </m.div>

        <m.div variants={varBounce('in')}>
          <ForbiddenIllustration sx={{ my: { xs: 5, sm: 10 } }} />
        </m.div>

        <Button component={RouterLink} href="/" size="large" variant="contained">
          Go to home
        </Button>
      </Container>
    </SimpleLayout>
  );
}

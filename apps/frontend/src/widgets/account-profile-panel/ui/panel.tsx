'use client';

import type { ReactNode } from 'react';

import { useState, useCallback } from 'react';

import Tab from '@mui/material/Tab';
import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Alert from '@mui/material/Alert';
import Container from '@mui/material/Container';
import CardHeader from '@mui/material/CardHeader';
import CardContent from '@mui/material/CardContent';

import { useTranslate } from 'src/shared/i18n/use-locales';
import { LoadingScreen } from 'src/shared/ui/loading-screen';
import { resolveServerAssetUrl } from 'src/shared/lib/asset-url';

import { useAuthContext } from 'src/entities/session';
import { useAccountProfile } from 'src/entities/user';
import { usePublicConfigs, passwordPolicyFromPublicConfigs } from 'src/entities/system';

import { ProfileCard } from './profile-card';
import { AvatarCropDialog } from './avatar-crop-dialog';
import { BasicProfileForm, PasswordProfileForm } from './profile-forms';

const BASIC_TAB = 'basic';
const PASSWORD_TAB = 'password';

export function AccountProfilePanel() {
  const { t } = useTranslate('admin');
  const { checkUserSession } = useAuthContext();
  const { data, error, isLoading, mutate } = useAccountProfile();
  const { data: publicConfigs, error: configError, isLoading: loadingConfig } = usePublicConfigs();
  const [tab, setTab] = useState(BASIC_TAB);
  const [avatarOpen, setAvatarOpen] = useState(false);

  const refreshProfile = useCallback(async () => {
    await mutate();
    await checkUserSession();
  }, [checkUserSession, mutate]);

  if (isLoading || loadingConfig) return <LoadingScreen portal={false} />;
  if (error)
    return (
      <Alert severity="error">
        {error instanceof Error ? error.message : t('messages.saveFailed')}
      </Alert>
    );
  if (configError)
    return (
      <Alert severity="error">
        {configError instanceof Error ? configError.message : t('messages.saveFailed')}
      </Alert>
    );
  if (!data) return <Alert severity="warning">{t('common.noData')}</Alert>;

  const passwordPolicy = passwordPolicyFromPublicConfigs(publicConfigs);

  return (
    <ProfileContainer
      tab={tab}
      profile={data}
      passwordPolicy={passwordPolicy}
      refreshProfile={refreshProfile}
      setTab={setTab}
      setAvatarOpen={setAvatarOpen}
    >
      <AvatarCropDialog
        open={avatarOpen}
        currentAvatar={resolveServerAssetUrl(data.user.avatar)}
        onClose={() => setAvatarOpen(false)}
        onUploaded={refreshProfile}
      />
    </ProfileContainer>
  );
}

type ProfileContainerProps = {
  tab: string;
  profile: NonNullable<ReturnType<typeof useAccountProfile>['data']>;
  passwordPolicy: ReturnType<typeof passwordPolicyFromPublicConfigs>;
  refreshProfile: () => Promise<void>;
  setTab: (tab: string) => void;
  setAvatarOpen: (open: boolean) => void;
  children: ReactNode;
};

function ProfileContainer(props: ProfileContainerProps) {
  const { tab, profile, passwordPolicy, refreshProfile, setTab, setAvatarOpen, children } = props;
  const { t } = useTranslate('admin');

  return (
    <Container maxWidth="xl" sx={{ py: 3 }}>
      <Grid container spacing={3}>
        <Grid size={{ xs: 12, md: 4 }}>
          <ProfileCard profile={profile} onChangeAvatar={() => setAvatarOpen(true)} />
        </Grid>
        <Grid size={{ xs: 12, md: 8 }}>
          <Card>
            <CardHeader title={t('profile.basicInfo')} />
            <Tabs value={tab} onChange={(_, value) => setTab(value)} sx={{ px: 3 }}>
              <Tab value={BASIC_TAB} label={t('profile.basicInfo')} />
              <Tab value={PASSWORD_TAB} label={t('profile.changePassword')} />
            </Tabs>
            <CardContent>
              <Box hidden={tab !== BASIC_TAB}>
                <BasicProfileForm user={profile.user} onSaved={refreshProfile} />
              </Box>
              <Box hidden={tab !== PASSWORD_TAB}>
                <PasswordProfileForm
                  username={profile.user.username}
                  passwordPolicy={passwordPolicy}
                />
              </Box>
            </CardContent>
          </Card>
        </Grid>
      </Grid>
      {children}
    </Container>
  );
}

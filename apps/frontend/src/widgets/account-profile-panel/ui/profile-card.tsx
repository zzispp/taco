'use client';

import type { AccountProfile } from 'src/entities/user';
import type { IconifyProps } from 'src/shared/ui/iconify';

import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import List from '@mui/material/List';
import Stack from '@mui/material/Stack';
import Avatar from '@mui/material/Avatar';
import Tooltip from '@mui/material/Tooltip';
import ListItem from '@mui/material/ListItem';
import Typography from '@mui/material/Typography';
import IconButton from '@mui/material/IconButton';
import CardHeader from '@mui/material/CardHeader';
import ListItemText from '@mui/material/ListItemText';
import ListItemIcon from '@mui/material/ListItemIcon';

import { Iconify } from 'src/shared/ui/iconify';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { fAdminDateTime } from 'src/shared/lib/admin-time';
import { resolveServerAssetUrl } from 'src/shared/lib/asset-url';

export type ProfileCardProps = {
  profile: AccountProfile;
  onChangeAvatar: () => void;
};

export function ProfileCard({ profile, onChangeAvatar }: ProfileCardProps) {
  const { t } = useTranslate('admin');
  const user = profile.user;
  const avatar = resolveServerAssetUrl(user.avatar);

  return (
    <Card>
      <CardHeader title={t('profile.personalInfo')} />
      <Stack alignItems="center" spacing={2} sx={{ px: 3, pb: 3 }}>
        <Box sx={{ position: 'relative' }}>
          <Avatar src={avatar} sx={{ width: 128, height: 128, fontSize: 48 }}>
            {user.nick_name.charAt(0).toUpperCase()}
          </Avatar>
          <Tooltip title={t('profile.changeAvatar')}>
            <IconButton
              color="primary"
              onClick={onChangeAvatar}
              sx={{ right: 0, bottom: 0, position: 'absolute', bgcolor: 'background.paper' }}
            >
              <Iconify icon="solar:camera-add-bold" />
            </IconButton>
          </Tooltip>
        </Box>
        <Box sx={{ textAlign: 'center' }}>
          <Typography variant="h6">{user.nick_name}</Typography>
          <Typography variant="body2" sx={{ color: 'text.secondary' }}>
            {user.email}
          </Typography>
        </Box>
      </Stack>
      <List disablePadding>
        {profileItems(profile, t).map((item) => (
          <ProfileItem key={item.label} {...item} />
        ))}
      </List>
    </Card>
  );
}

function ProfileItem({ icon, label, value }: ProfileItemData) {
  return (
    <ListItem divider>
      <ListItemIcon>
        <Iconify icon={icon} width={20} />
      </ListItemIcon>
      <ListItemText
        primary={label}
        secondary={value || '-'}
        slotProps={{ secondary: { noWrap: true } }}
      />
    </ListItem>
  );
}

function profileItems(
  profile: AccountProfile,
  t: ReturnType<typeof useTranslate>['t']
): ProfileItemData[] {
  const user = profile.user;
  return [
    { icon: 'solar:user-rounded-bold', label: t('common.username'), value: user.username },
    { icon: 'solar:phone-bold', label: t('fields.phone'), value: user.phonenumber ?? '' },
    { icon: 'solar:letter-bold', label: t('common.email'), value: user.email },
    {
      icon: 'solar:add-folder-bold',
      label: t('profile.deptAndPost'),
      value: joinGroup(profile.dept_name, profile.post_group),
    },
    {
      icon: 'solar:users-group-rounded-bold',
      label: t('profile.roleGroup'),
      value: profile.role_group,
    },
    {
      icon: 'solar:calendar-date-bold',
      label: t('fields.createTime'),
      value: fAdminDateTime(user.create_time),
    },
  ];
}

function joinGroup(deptName: string | null, postGroup: string) {
  return [deptName, postGroup].filter(Boolean).join(' / ');
}

type ProfileItemData = {
  icon: IconifyProps['icon'];
  label: string;
  value: string;
};

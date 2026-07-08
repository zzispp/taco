'use client';

import type { PasswordPolicy } from 'src/entities/system';
import type { SystemUser, ProfileInput } from 'src/entities/user';

import { useState, useEffect, useCallback } from 'react';

import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import Button from '@mui/material/Button';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import Typography from '@mui/material/Typography';

import { toast } from 'src/shared/ui/snackbar';
import { useTranslate } from 'src/shared/i18n/use-locales';

import { createPasswordSchema } from 'src/entities/session';

import { updateAccountProfile, changeAccountPassword } from 'src/features/user-profile';

const EMAIL_PATTERN = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const PHONE_PATTERN = /^1[3-9]\d{9}$/;

export function BasicProfileForm({ user, onSaved }: BasicProfileFormProps) {
  const { t } = useTranslate('admin');
  const [form, setForm] = useState<ProfileInput>(profileForm(user));
  const [loading, setLoading] = useState(false);

  useEffect(() => setForm(profileForm(user)), [user]);

  const updateField = useCallback((field: keyof ProfileInput, value: string) => {
    setForm((current) => ({
      ...current,
      [field]: field === 'phonenumber' && !value.trim() ? null : value,
    }));
  }, []);

  const handleSubmit = useCallback(async () => {
    const error = profileFormError(form, t);
    if (error) {
      toast.error(error);
      return;
    }
    setLoading(true);
    try {
      await updateAccountProfile(form);
      await onSaved();
      toast.success(t('messages.saved'));
    } catch (caught) {
      toast.error(caught instanceof Error ? caught.message : t('messages.saveFailed'));
    } finally {
      setLoading(false);
    }
  }, [form, onSaved, t]);

  return (
    <Stack spacing={3} sx={{ maxWidth: 560 }}>
      <TextField
        label={t('fields.nickName')}
        value={form.nick_name}
        onChange={(event) => updateField('nick_name', event.target.value)}
      />
      <TextField
        type="tel"
        label={t('fields.phone')}
        value={form.phonenumber ?? ''}
        onChange={(event) => updateField('phonenumber', event.target.value)}
      />
      <TextField
        type="email"
        label={t('common.email')}
        value={form.email}
        onChange={(event) => updateField('email', event.target.value)}
      />
      <TextField
        select
        label={t('fields.sex')}
        value={form.sex}
        onChange={(event) => updateField('sex', event.target.value)}
      >
        <MenuItem value="0">{t('common.male')}</MenuItem>
        <MenuItem value="1">{t('common.female')}</MenuItem>
        <MenuItem value="2">{t('common.unknown')}</MenuItem>
      </TextField>
      <Box>
        <Button variant="contained" loading={loading} onClick={handleSubmit}>
          {t('common.save')}
        </Button>
      </Box>
    </Stack>
  );
}

export function PasswordProfileForm({ username, passwordPolicy }: PasswordProfileFormProps) {
  const { t } = useTranslate('admin');
  const [form, setForm] = useState(passwordForm());
  const [loading, setLoading] = useState(false);

  const updateField = useCallback((field: keyof PasswordFormState, value: string) => {
    setForm((current) => ({ ...current, [field]: value }));
  }, []);

  const handleSubmit = useCallback(async () => {
    const error = passwordFormError(form, t, username, passwordPolicy);
    if (error) {
      toast.error(error);
      return;
    }
    setLoading(true);
    try {
      await changeAccountPassword(form.old_password, form.new_password);
      setForm(passwordForm());
      toast.success(t('messages.saved'));
    } catch (caught) {
      toast.error(caught instanceof Error ? caught.message : t('messages.saveFailed'));
    } finally {
      setLoading(false);
    }
  }, [form, passwordPolicy, t, username]);

  return (
    <Stack spacing={3} sx={{ maxWidth: 560 }}>
      <TextField
        type="password"
        label={t('profile.oldPassword')}
        value={form.old_password}
        onChange={(event) => updateField('old_password', event.target.value)}
      />
      <TextField
        type="password"
        label={t('fields.newPassword')}
        value={form.new_password}
        onChange={(event) => updateField('new_password', event.target.value)}
      />
      <TextField
        type="password"
        label={t('profile.confirmPassword')}
        value={form.confirm_password}
        onChange={(event) => updateField('confirm_password', event.target.value)}
      />
      <Stack direction="row" spacing={2} alignItems="center">
        <Button variant="contained" loading={loading} onClick={handleSubmit}>
          {t('common.save')}
        </Button>
        <Typography variant="caption" sx={{ color: 'text.secondary' }}>
          {passwordRuleText(t, passwordPolicy)}
        </Typography>
      </Stack>
    </Stack>
  );
}

function profileForm(user: SystemUser): ProfileInput {
  return {
    nick_name: user.nick_name,
    phonenumber: user.phonenumber,
    email: user.email,
    sex: user.sex,
  };
}

function passwordForm(): PasswordFormState {
  return { old_password: '', new_password: '', confirm_password: '' };
}

function profileFormError(form: ProfileInput, t: ReturnType<typeof useTranslate>['t']) {
  if (!form.nick_name.trim() || !form.email.trim()) return t('profile.requiredProfileFields');
  if (!EMAIL_PATTERN.test(form.email.trim())) return t('profile.invalidEmail');
  if (form.phonenumber?.trim() && !PHONE_PATTERN.test(form.phonenumber.trim())) {
    return t('profile.invalidPhone');
  }
  return '';
}

function passwordFormError(
  form: PasswordFormState,
  t: ReturnType<typeof useTranslate>['t'],
  username: string,
  policy?: PasswordPolicy
) {
  if (!form.old_password || !form.new_password || !form.confirm_password)
    return t('profile.passwordRequired');
  const parsed = createPasswordSchema(validationMessages(t), policy, username).safeParse(form.new_password);
  if (!parsed.success) return parsed.error.issues[0]?.message ?? passwordRuleText(t, policy);
  if (form.old_password === form.new_password) return t('profile.passwordSame');
  if (form.new_password !== form.confirm_password) return t('profile.passwordMismatch');
  return '';
}

function validationMessages(t: ReturnType<typeof useTranslate>['t']) {
  return {
    usernameLength: (min: number, max: number) => t('profile.usernameRuleDynamic', { min, max }),
    usernamePattern: t('profile.usernamePattern'),
    passwordLength: (min: number, max: number) => t('profile.passwordRuleDynamic', { min, max }),
    passwordLetterRequired: t('profile.passwordLetterRequired'),
    passwordNumberRequired: t('profile.passwordNumberRequired'),
    passwordSymbolRequired: t('profile.passwordSymbolRequired'),
    passwordContainsUsername: t('profile.passwordContainsUsername'),
    emailRequired: t('profile.emailRequired'),
    emailInvalid: t('profile.invalidEmail'),
    identifierRequired: t('profile.identifierRequired'),
    identifierInvalid: t('profile.identifierInvalid'),
  };
}

function passwordRuleText(t: ReturnType<typeof useTranslate>['t'], policy?: PasswordPolicy) {
  if (!policy) return t('profile.passwordRule');
  return t('profile.passwordRuleDynamic', { min: policy.min_length, max: policy.max_length });
}

type BasicProfileFormProps = {
  user: SystemUser;
  onSaved: () => Promise<void>;
};

type PasswordProfileFormProps = {
  username: string;
  passwordPolicy?: PasswordPolicy;
};

type PasswordFormState = {
  old_password: string;
  new_password: string;
  confirm_password: string;
};

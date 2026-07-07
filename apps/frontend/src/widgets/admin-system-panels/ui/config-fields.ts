import type { TranslateFn } from 'src/shared/i18n';
import type { ConfigItem, ConfigInput } from 'src/entities/system';

export function configFields(t: TranslateFn) {
  return [
    { key: 'config_name' as const, label: t('fields.configName'), width: 180, ellipsis: true },
    { key: 'config_key' as const, label: t('fields.configKey'), width: 260, ellipsis: true, disabled: builtInFieldDisabled },
    { key: 'config_value' as const, label: t('fields.configValue'), width: 360, ellipsis: true },
    { key: 'config_type' as const, label: t('fields.configType'), type: 'select' as const, width: 120, options: configTypeOptions(t), disabled: builtInFieldDisabled },
    { key: 'public_read' as const, label: t('fields.publicRead'), type: 'boolean' as const, width: 120, disabled: publicReadDisabled },
    { key: 'remark' as const, label: t('common.remark'), type: 'textarea' as const, width: 280, ellipsis: true },
    { key: 'create_time' as const, label: t('fields.createTime'), width: 190, format: 'dateTime' as const, hiddenInForm: true },
  ];
}

export function configFilterFields(t: TranslateFn) {
  return [
    { key: 'config_name', label: t('fields.configName') },
    { key: 'config_key', label: t('fields.configKey') },
    { key: 'config_type', label: t('fields.configType'), type: 'select' as const, options: allConfigTypeOptions(t) },
    { key: 'begin_time', label: t('fields.beginTime'), type: 'date' as const },
    { key: 'end_time', label: t('fields.endTime'), type: 'date' as const },
  ];
}

export function toConfigFilters(values: Record<string, string>) {
  return {
    config_name: values.config_name ?? '',
    config_key: values.config_key ?? '',
    config_type: values.config_type ?? '',
    begin_time: values.begin_time ?? '',
    end_time: values.end_time ?? '',
  };
}

export function isConfigSelectable(row: ConfigItem) {
  return row.config_type !== 'Y';
}

export function normalizeConfigInput(input: ConfigInput): ConfigInput {
  if (input.config_key === 'sys.user.initPassword') return { ...input, public_read: false };
  return input;
}

function configTypeOptions(t: TranslateFn) {
  return [
    { value: 'Y', label: t('common.yes') },
    { value: 'N', label: t('common.no') },
  ];
}

function allConfigTypeOptions(t: TranslateFn) {
  return [{ value: '', label: t('common.all') }, ...configTypeOptions(t)];
}

function builtInFieldDisabled({ editing }: { editing: Record<string, unknown> | null }) {
  return editing?.config_type === 'Y';
}

function publicReadDisabled({ form, editing }: { form: Record<string, unknown>; editing: Record<string, unknown> | null }) {
  const key = String(form.config_key || editing?.config_key || '');
  return key === 'sys.user.initPassword';
}

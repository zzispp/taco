import type { TranslateFn } from 'src/shared/i18n';

export function postFields(t: TranslateFn) {
  return [
    { key: 'post_name' as const, label: t('fields.postName') },
    { key: 'post_code' as const, label: t('fields.postCode') },
    { key: 'post_sort' as const, label: t('fields.postSort'), type: 'number' as const },
    { key: 'status' as const, label: t('common.status'), type: 'select' as const, options: statusOptions(t) },
    { key: 'remark' as const, label: t('common.remark'), type: 'textarea' as const },
    { key: 'create_time' as const, label: t('fields.createTime'), format: 'dateTime' as const, hiddenInForm: true },
  ];
}

export function postFilterFields(t: TranslateFn) {
  return [
    { key: 'post_name', label: t('fields.postName') },
    { key: 'post_code', label: t('fields.postCode') },
    { key: 'status', label: t('common.status'), type: 'select' as const, options: allStatusOptions(t) },
  ];
}

export function toPostFilters(values: Record<string, string>) {
  return {
    post_name: values.post_name ?? '',
    post_code: values.post_code ?? '',
    status: values.status ?? '',
  };
}

function statusOptions(t: TranslateFn) {
  return [
    { value: '0', label: t('common.enabled') },
    { value: '1', label: t('common.disabled') },
  ];
}

function allStatusOptions(t: TranslateFn) {
  return [{ value: '', label: t('common.all') }, ...statusOptions(t)];
}

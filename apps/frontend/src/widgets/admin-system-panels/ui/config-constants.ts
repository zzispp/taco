import type { ConfigInput } from 'src/entities/system';

export const DEFAULT_CONFIG_INPUT: ConfigInput = {
  config_name: '',
  config_key: '',
  config_value: '',
  config_type: 'N',
  public_read: false,
  remark: '',
};

export const DEFAULT_CONFIG_FILTERS = {
  config_name: '',
  config_key: '',
  config_type: '',
  public_read: '',
  begin_time: '',
  end_time: '',
};

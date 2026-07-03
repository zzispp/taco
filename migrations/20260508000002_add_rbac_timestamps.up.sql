ALTER TABLE roles ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ;
ALTER TABLE roles ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
UPDATE roles SET created_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP;
ALTER TABLE roles ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE roles ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE api_permissions ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ;
ALTER TABLE api_permissions ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
UPDATE api_permissions SET created_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP;
ALTER TABLE api_permissions ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE api_permissions ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE menu_sections ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ;
ALTER TABLE menu_sections ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
UPDATE menu_sections SET created_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP;
ALTER TABLE menu_sections ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE menu_sections ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE menu_items ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ;
ALTER TABLE menu_items ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
UPDATE menu_items SET created_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP;
ALTER TABLE menu_items ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE menu_items ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE role_api_permissions ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ;
ALTER TABLE role_api_permissions ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
UPDATE role_api_permissions SET created_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP;
ALTER TABLE role_api_permissions ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE role_api_permissions ALTER COLUMN updated_at SET NOT NULL;

ALTER TABLE role_menu_permissions ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ;
ALTER TABLE role_menu_permissions ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ;
UPDATE role_menu_permissions SET created_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP;
ALTER TABLE role_menu_permissions ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE role_menu_permissions ALTER COLUMN updated_at SET NOT NULL;

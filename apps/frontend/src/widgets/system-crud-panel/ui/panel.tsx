'use client';

import type { CrudField, CrudFilter, CrudPanelProps, ActionIconProps } from './types';

import { AdminBreadcrumbs } from 'src/widgets/admin-common';
import { DashboardContent } from 'src/widgets/dashboard-shell';

import { ActionIcon } from './action-icon';
import { CrudTableSection } from './table-section';
import { CrudDialogSection } from './dialog-section';
import { CrudToolbarSection } from './toolbar-section';
import { useSystemCrudController } from './controller';

export type { CrudField, CrudFilter, CrudPanelProps, ActionIconProps };
export { ActionIcon };

export function SystemCrudPanel<
  T extends Record<string, unknown>,
  I extends Record<string, unknown>,
>(props: CrudPanelProps<T, I>) {
  const controller = useSystemCrudController(props);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={props.title}
        action={
          <CrudToolbarSection
            addLabel={props.addLabel}
            toolbarAction={props.toolbarAction}
            controller={controller}
          />
        }
      />
      <CrudTableSection props={props} controller={controller} />
      <CrudDialogSection props={props} controller={controller} />
    </DashboardContent>
  );
}

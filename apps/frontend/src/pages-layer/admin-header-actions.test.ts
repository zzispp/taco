import { readFileSync } from 'node:fs';
import { it, expect, describe } from 'vitest';

const HEADER_ACTION_CASES = [
  headerActionCase({
    name: '定时任务',
    component: 'SchedulerToolbar',
    headerOwnerPath: '../widgets/admin-scheduler-panel/ui/panel.tsx',
    panelDir: '../widgets/admin-scheduler-panel',
  }),
  headerActionCase({
    name: '调度日志',
    component: 'JobLogToolbar',
    headerOwnerPath: '../widgets/admin-job-logs-panel/ui/panel.tsx',
    panelDir: '../widgets/admin-job-logs-panel',
  }),
  headerActionCase({
    name: '操作日志',
    component: 'OperationLogToolbar',
    headerOwnerPath: './operation-logs/ui/page.tsx',
    panelDir: '../widgets/admin-operation-logs-panel',
  }),
  headerActionCase({
    name: '登录日志',
    component: 'LoginLogToolbar',
    headerOwnerPath: './login-logs/ui/page.tsx',
    panelDir: '../widgets/admin-login-logs-panel',
  }),
  headerActionCase({
    name: '系统日志',
    component: 'SystemLogToolbar',
    headerOwnerPath: './system-logs/ui/page.tsx',
    panelDir: '../widgets/admin-system-logs-panel',
  }),
] as const;

describe('admin header action placement', () => {
  it.each(HEADER_ACTION_CASES)('$name renders its actions through AdminBreadcrumbs', (entry) => {
    const headerOwner = readFileSync(entry.headerOwnerUrl, 'utf8');
    const toolbar = readFileSync(entry.toolbarUrl, 'utf8');
    const invocation = `<${entry.component} controller={controller} />`;

    expect(headerOwner).toContain(`action={${invocation}}`);
    expect(headerOwner.split(invocation)).toHaveLength(2);
    expect(toolbar).not.toContain('sx={{ mb: 2 }}');
  });

  it.each(HEADER_ACTION_CASES.filter((entry) => entry.panelUrl.href !== entry.headerOwnerUrl.href))(
    '$name no longer renders an independent panel toolbar',
    (entry) => {
      const panel = readFileSync(entry.panelUrl, 'utf8');

      expect(panel).not.toContain(`<${entry.component} controller={controller} />`);
    }
  );
});

function headerActionCase(options: HeaderActionCaseOptions) {
  return {
    name: options.name,
    component: options.component,
    headerOwnerUrl: new URL(options.headerOwnerPath, import.meta.url),
    panelUrl: new URL(`${options.panelDir}/ui/panel.tsx`, import.meta.url),
    toolbarUrl: new URL(`${options.panelDir}/ui/toolbar.tsx`, import.meta.url),
  } as const;
}

type HeaderActionCaseOptions = Readonly<{
  name: string;
  component: string;
  headerOwnerPath: string;
  panelDir: string;
}>;

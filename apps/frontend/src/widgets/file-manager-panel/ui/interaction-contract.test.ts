import { readFileSync } from 'node:fs';
import { it, expect, describe } from 'vitest';

import cnAdminFile from 'src/shared/i18n/langs/cn/admin-file.json';
import twAdminFile from 'src/shared/i18n/langs/tw/admin-file.json';

const DETAILS_DRAWER_URL = new URL('./details-drawer.tsx', import.meta.url);
const ENTRY_ACTIONS_URL = new URL('./entry-actions.tsx', import.meta.url);
const ENTRY_ITEM_URL = new URL('./entry-item.tsx', import.meta.url);
const FILE_MANAGER_CONTEXT_MENU_URL = new URL('./file-manager-context-menu.tsx', import.meta.url);
const FILE_MANAGER_CONTEXT_MENU_ACTIONS_URL = new URL(
  './file-manager-context-menu-actions.ts',
  import.meta.url
);
const ENTRY_COLLECTION_URL = new URL('./entry-collection.tsx', import.meta.url);
const MOVE_DIALOG_URL = new URL('./move-dialog.tsx', import.meta.url);
const MOVE_BROWSER_URL = new URL('./move-destination-browser.tsx', import.meta.url);
const TOOLBAR_URL = new URL('./toolbar.tsx', import.meta.url);
const DIRECTORY_NAVIGATION_URL = new URL('./directory-navigation.tsx', import.meta.url);
const PANEL_URL = new URL('./panel.tsx', import.meta.url);

describe('file manager interaction contracts', () => {
  it('keeps an invisible backdrop so outside clicks close the details drawer', () => {
    const source = readFileSync(DETAILS_DRAWER_URL, 'utf8');

    expect(source).toContain('backdrop: { invisible: true }');
    expect(source).not.toContain('hideBackdrop');
  });

  it('keeps double-click handlers on both file entry views', () => {
    const source = readFileSync(ENTRY_ITEM_URL, 'utf8');

    expect(source).toContain('onContextMenu={(event) => onContextMenu(event, entry)}');
    expect(source.match(/onDoubleClick=\{\(\) =>/g)).toHaveLength(2);
  });

  it('uses a MUI anchor-position menu for right-click actions', () => {
    const source = readFileSync(FILE_MANAGER_CONTEXT_MENU_URL, 'utf8');

    expect(source).toContain("import Menu from '@mui/material/Menu'");
    expect(source).toContain('anchorReference="anchorPosition"');
  });

  it('opens directory actions from unoccupied asset-table space without replacing row actions', () => {
    const collection = readFileSync(ENTRY_COLLECTION_URL, 'utf8');
    const contextMenu = readFileSync(FILE_MANAGER_CONTEXT_MENU_URL, 'utf8');

    expect(collection).toContain('onContextMenu={contextMenu.onDirectoryContextMenu}');
    expect(collection).toContain('<FileManagerContextMenus');
    expect(contextMenu).toContain('preventBrowserContextMenu(event)');
  });

  it('gates every details affordance with the shared query policy', () => {
    const actions = readFileSync(ENTRY_ACTIONS_URL, 'utf8');
    const contextMenu = readFileSync(FILE_MANAGER_CONTEXT_MENU_ACTIONS_URL, 'utf8');
    const drawer = readFileSync(DETAILS_DRAWER_URL, 'utf8');

    expect(actions).toContain('canViewFileEntryDetails');
    expect(contextMenu).toContain('canViewFileEntryDetails');
    expect(drawer).toContain('canViewFileEntryDetails');
  });

  it('localizes the details heading in both Chinese resources', () => {
    expect(cnAdminFile.file.info).toBe('信息');
    expect(twAdminFile.file.info).toBe('資訊');
  });

  it('keeps parent navigation in the toolbar outside collection early returns', () => {
    const toolbar = readFileSync(TOOLBAR_URL, 'utf8');
    const directoryNavigation = readFileSync(DIRECTORY_NAVIGATION_URL, 'utf8');
    const panel = readFileSync(PANEL_URL, 'utf8');

    expect(toolbar).toContain('<FileManagerDirectoryNavigation controller={controller} />');
    expect(directoryNavigation).toContain("import IconButton from '@mui/material/IconButton'");
    expect(directoryNavigation).toContain("import Breadcrumbs from '@mui/material/Breadcrumbs'");
    expect(directoryNavigation).toContain('controller.actions.goToParentFolder');
    expect(directoryNavigation).toContain('disabled={atRoot || !ready}');
    expect(directoryNavigation).toContain('controller.actions.goToDirectory(item.directoryTrail)');
    expect(panel).not.toContain('controller.actions.closeFolder');
  });

  it('uses a hierarchical MUI folder browser for move destinations', () => {
    const dialog = readFileSync(MOVE_DIALOG_URL, 'utf8');
    const browser = readFileSync(MOVE_BROWSER_URL, 'utf8');

    expect(dialog).not.toContain('<TextField');
    expect(dialog).toContain("t('file.actions.moveHere')");
    expect(browser).toContain("import Breadcrumbs from '@mui/material/Breadcrumbs'");
    expect(browser).toContain("import ListItemButton from '@mui/material/ListItemButton'");
    expect(browser).toContain('browser.parentId');
    expect(browser).toContain('folders.map');
  });

  it('provides right-click commands for both folders and unoccupied move-browser space', () => {
    const browser = readFileSync(MOVE_BROWSER_URL, 'utf8');
    const contextMenu = readFileSync(
      new URL('./move-destination-context-menu.tsx', import.meta.url),
      'utf8'
    );

    expect(browser).toMatch(
      /<Scrollbar[\s\S]*?onContextMenu=\{contextMenus\.onDirectoryContextMenu\}/
    );
    expect(browser).toContain('onFolderContextMenu');
    expect(browser).toContain('<MoveDirectoryContextMenu');
    expect(browser).toContain('<MoveFolderContextMenu');
    expect(contextMenu).toContain('const [contextMenu, setContextMenu] = useState');
    expect(browser).not.toContain('const directory = useEntryContextMenu()');
    expect(contextMenu).toContain("import Menu from '@mui/material/Menu'");
  });
});

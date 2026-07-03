'use client';

import type { TableHeadCellProps } from 'src/shared/ui/table';
import type {
  MenuSection,
  MenuItemInput,
  MenuSectionInput,
  MenuItem as RbacMenuItem,
} from 'src/entities/menu';

import { useMemo, useState, useCallback } from 'react';

import Box from '@mui/material/Box';
import Tab from '@mui/material/Tab';
import Card from '@mui/material/Card';
import Tabs from '@mui/material/Tabs';
import Table from '@mui/material/Table';
import Button from '@mui/material/Button';
import Tooltip from '@mui/material/Tooltip';
import MenuItem from '@mui/material/MenuItem';
import TableRow from '@mui/material/TableRow';
import TableBody from '@mui/material/TableBody';
import TableCell from '@mui/material/TableCell';
import IconButton from '@mui/material/IconButton';

import { Label } from 'src/shared/ui/label';
import { toast } from 'src/shared/ui/snackbar';
import { Iconify } from 'src/shared/ui/iconify';
import { Scrollbar } from 'src/shared/ui/scrollbar';
import { useTranslate } from 'src/shared/i18n/use-locales';
import { ConfirmDialog } from 'src/shared/ui/custom-dialog';
import { useTable, TableNoData, TablePaginationCustom } from 'src/shared/ui/table';
import {
  AddButton,
  SwitchRow,
  TextFieldRow,
  EnabledLabel,
  AdminBreadcrumbs,
  ManagementDialog,
  TableLoadingRows,
  ManagementTableHead,
} from 'src/shared/ui/admin';

import {
  useMenuItems,
  useMenuSections,
  NAV_ICON_OPTIONS,
  translatedMenuItem,
  translatedMenuSection,
} from 'src/entities/menu';

import {
  createMenuItem,
  deleteMenuItem,
  updateMenuItem,
  createMenuSection,
  deleteMenuSection,
  updateMenuSection,
} from 'src/features/menu-management/api';

import { DashboardContent } from 'src/widgets/dashboard-shell';

// ----------------------------------------------------------------------

const DEFAULT_SECTION: MenuSectionInput = {
  code: '',
  subheader: '',
  sort_order: 0,
  enabled: true,
};

const DEFAULT_ITEM: MenuItemInput = {
  section_id: '',
  parent_id: null,
  code: '',
  title: '',
  path: '',
  icon: 'icon.menu',
  caption: null,
  deep_match: true,
  sort_order: 0,
  enabled: true,
};

// ----------------------------------------------------------------------

export function MenuManagementView() {
  const { t } = useTranslate('admin');
  const [tab, setTab] = useState('items');

  const sectionTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });
  const itemTable = useTable({ defaultRowsPerPage: 10, defaultOrderBy: 'sort_order' });

  const sections = useMenuSections(sectionTable.page, sectionTable.rowsPerPage);
  const allSections = useMenuSections(0, 100);
  const items = useMenuItems(itemTable.page, itemTable.rowsPerPage);
  const allItems = useMenuItems(0, 100);
  const sectionHead = useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'subheader', label: t('fields.subheader'), width: 240 },
      { id: 'code', label: t('common.code'), width: 240 },
      { id: 'sort_order', label: t('common.sort'), width: 100 },
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: '', width: 96 },
    ],
    [t]
  );
  const itemHead = useMemo<TableHeadCellProps[]>(
    () => [
      { id: 'title', label: t('common.title'), width: 220 },
      { id: 'code', label: t('common.code'), width: 220 },
      { id: 'path', label: t('common.path') },
      { id: 'section', label: t('common.section'), width: 180 },
      { id: 'sort_order', label: t('common.sort'), width: 100 },
      { id: 'enabled', label: t('common.status'), width: 120 },
      { id: '', width: 96 },
    ],
    [t]
  );

  const [sectionForm, setSectionForm] = useState<MenuSectionInput>(DEFAULT_SECTION);
  const [itemForm, setItemForm] = useState<MenuItemInput>(DEFAULT_ITEM);
  const [editingSection, setEditingSection] = useState<MenuSection | null>(null);
  const [editingItem, setEditingItem] = useState<RbacMenuItem | null>(null);
  const [creatingSection, setCreatingSection] = useState(false);
  const [creatingItem, setCreatingItem] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [deleteSectionTarget, setDeleteSectionTarget] = useState<MenuSection | null>(null);
  const [deleteItemTarget, setDeleteItemTarget] = useState<RbacMenuItem | null>(null);

  const sectionNameById = useMemo(
    () => new Map(allSections.items.map((section) => [section.id, translatedMenuSection(section, t)])),
    [allSections.items, t]
  );

  const openCreateSection = useCallback(() => {
    setEditingSection(null);
    setCreatingSection(true);
    setSectionForm({ ...DEFAULT_SECTION });
  }, []);

  const openEditSection = useCallback((section: MenuSection) => {
    setEditingSection(section);
    setSectionForm({
      code: section.code,
      subheader: section.subheader,
      sort_order: section.sort_order,
      enabled: section.enabled,
    });
  }, []);

  const closeSectionDialog = useCallback(() => {
    setEditingSection(null);
    setCreatingSection(false);
    setSectionForm(DEFAULT_SECTION);
  }, []);

  const openCreateItem = useCallback(() => {
    setEditingItem(null);
    setCreatingItem(true);
    setItemForm({
      ...DEFAULT_ITEM,
      section_id: allSections.items[0]?.id ?? '',
    });
  }, [allSections.items]);

  const openEditItem = useCallback((item: RbacMenuItem) => {
    setEditingItem(item);
    setItemForm({
      section_id: item.section_id,
      parent_id: item.parent_id,
      code: item.code,
      title: item.title,
      path: item.path,
      icon: item.icon,
      caption: item.caption,
      deep_match: item.deep_match,
      sort_order: item.sort_order,
      enabled: item.enabled,
    });
  }, []);

  const closeItemDialog = useCallback(() => {
    setEditingItem(null);
    setCreatingItem(false);
    setItemForm(DEFAULT_ITEM);
  }, []);

  const submitSection = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editingSection) {
        await updateMenuSection(editingSection.id, sectionForm);
        toast.success(t('messages.menuSectionUpdated'));
      } else {
        await createMenuSection(sectionForm);
        toast.success(t('messages.menuSectionCreated'));
      }
      closeSectionDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeSectionDialog, editingSection, sectionForm, t]);

  const submitItem = useCallback(async () => {
    setSubmitting(true);
    try {
      if (editingItem) {
        await updateMenuItem(editingItem.id, itemForm);
        toast.success(t('messages.menuItemUpdated'));
      } else {
        await createMenuItem(itemForm);
        toast.success(t('messages.menuItemCreated'));
      }
      closeItemDialog();
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.saveFailed'));
    } finally {
      setSubmitting(false);
    }
  }, [closeItemDialog, editingItem, itemForm, t]);

  const confirmDeleteSection = useCallback(async () => {
    if (!deleteSectionTarget) return;

    try {
      await deleteMenuSection(deleteSectionTarget.id);
      toast.success(t('messages.menuSectionDeleted'));
      setDeleteSectionTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteSectionTarget, t]);

  const confirmDeleteItem = useCallback(async () => {
    if (!deleteItemTarget) return;

    try {
      await deleteMenuItem(deleteItemTarget.id);
      toast.success(t('messages.menuItemDeleted'));
      setDeleteItemTarget(null);
    } catch (error) {
      toast.error(error instanceof Error ? error.message : t('messages.deleteFailed'));
    }
  }, [deleteItemTarget, t]);

  return (
    <DashboardContent>
      <AdminBreadcrumbs
        heading={t('pages.menuManagement')}
        action={
          <AddButton onClick={tab === 'items' ? openCreateItem : openCreateSection}>
            {tab === 'items' ? t('actions.addMenuItem') : t('actions.addSection')}
          </AddButton>
        }
      />

      <Card>
        <Tabs value={tab} onChange={(event, value) => setTab(value)} sx={{ px: 2.5 }}>
          <Tab value="items" label={t('common.menus')} />
          <Tab value="sections" label={t('common.section')} />
        </Tabs>

        {tab === 'items' ? (
          <>
            <Scrollbar>
              <Table sx={{ minWidth: 1100 }}>
                <ManagementTableHead head={itemHead} />
                <TableBody>
                  {items.isLoading ? (
                    <TableLoadingRows head={itemHead} rows={itemTable.rowsPerPage} />
                  ) : (
                    items.items.map((row) => (
                      <TableRow key={row.id} hover>
                        <TableCell>{translatedMenuItem(row, t)}</TableCell>
                        <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
                        <TableCell sx={{ fontFamily: 'monospace' }}>{row.path}</TableCell>
                        <TableCell>{sectionNameById.get(row.section_id) ?? row.section_id}</TableCell>
                        <TableCell>{row.sort_order}</TableCell>
                        <TableCell>
                          <EnabledLabel enabled={row.enabled} />
                        </TableCell>
                        <TableCell align="right">
                          <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                            <Tooltip title={t('common.edit')}>
                              <IconButton onClick={() => openEditItem(row)}>
                                <Iconify icon="solar:pen-bold" />
                              </IconButton>
                            </Tooltip>
                            <Tooltip title={t('common.delete')}>
                              <IconButton color="error" onClick={() => setDeleteItemTarget(row)}>
                                <Iconify icon="solar:trash-bin-trash-bold" />
                              </IconButton>
                            </Tooltip>
                          </Box>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                  <TableNoData
                    title={t('common.noData')}
                    notFound={!items.isLoading && items.items.length === 0}
                  />
                </TableBody>
              </Table>
            </Scrollbar>
            <TablePaginationCustom
              page={itemTable.page}
              count={items.total}
              rowsPerPage={itemTable.rowsPerPage}
              onPageChange={itemTable.onChangePage}
              onRowsPerPageChange={itemTable.onChangeRowsPerPage}
            />
          </>
        ) : (
          <>
            <Scrollbar>
              <Table sx={{ minWidth: 820 }}>
                <ManagementTableHead head={sectionHead} />
                <TableBody>
                  {sections.isLoading ? (
                    <TableLoadingRows head={sectionHead} rows={sectionTable.rowsPerPage} />
                  ) : (
                    sections.items.map((row) => (
                      <TableRow key={row.id} hover>
                        <TableCell>{translatedMenuSection(row, t)}</TableCell>
                        <TableCell sx={{ fontFamily: 'monospace' }}>{row.code}</TableCell>
                        <TableCell>{row.sort_order}</TableCell>
                        <TableCell>
                          <EnabledLabel enabled={row.enabled} />
                        </TableCell>
                        <TableCell align="right">
                          <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
                            <Tooltip title={t('common.edit')}>
                              <IconButton onClick={() => openEditSection(row)}>
                                <Iconify icon="solar:pen-bold" />
                              </IconButton>
                            </Tooltip>
                            <Tooltip title={t('common.delete')}>
                              <IconButton color="error" onClick={() => setDeleteSectionTarget(row)}>
                                <Iconify icon="solar:trash-bin-trash-bold" />
                              </IconButton>
                            </Tooltip>
                          </Box>
                        </TableCell>
                      </TableRow>
                    ))
                  )}
                  <TableNoData
                    title={t('common.noData')}
                    notFound={!sections.isLoading && sections.items.length === 0}
                  />
                </TableBody>
              </Table>
            </Scrollbar>
            <TablePaginationCustom
              page={sectionTable.page}
              count={sections.total}
              rowsPerPage={sectionTable.rowsPerPage}
              onPageChange={sectionTable.onChangePage}
              onRowsPerPageChange={sectionTable.onChangeRowsPerPage}
            />
          </>
        )}
      </Card>

      <ManagementDialog
        open={creatingSection || !!editingSection}
        title={editingSection ? t('dialogs.editMenuSection') : t('dialogs.createMenuSection')}
        submitting={submitting}
        onClose={closeSectionDialog}
        onSubmit={submitSection}
      >
        <TextFieldRow
          required
          label={t('fields.subheader')}
          value={sectionForm.subheader}
          onChange={(value) => setSectionForm((current) => ({ ...current, subheader: value }))}
        />
        <TextFieldRow
          required
          label={t('common.code')}
          value={sectionForm.code}
          onChange={(value) => setSectionForm((current) => ({ ...current, code: value }))}
        />
        <TextFieldRow
          type="number"
          label={t('common.sortOrder')}
          value={sectionForm.sort_order}
          onChange={(value) => setSectionForm((current) => ({ ...current, sort_order: Number(value) }))}
        />
        <SwitchRow
          label={t('common.enabled')}
          checked={sectionForm.enabled}
          onChange={(enabled) => setSectionForm((current) => ({ ...current, enabled }))}
        />
      </ManagementDialog>

      <ManagementDialog
        open={creatingItem || !!editingItem}
        title={editingItem ? t('dialogs.editMenuItem') : t('dialogs.createMenuItem')}
        submitting={submitting}
        onClose={closeItemDialog}
        onSubmit={submitItem}
      >
        <TextFieldRow
          required
          select
          label={t('common.section')}
          value={itemForm.section_id}
          onChange={(value) => setItemForm((current) => ({ ...current, section_id: value }))}
        >
          {allSections.items.map((section) => (
            <MenuItem key={section.id} value={section.id}>
              {translatedMenuSection(section, t)}
            </MenuItem>
          ))}
        </TextFieldRow>
        <TextFieldRow
          select
          label={t('fields.parentItem')}
          value={itemForm.parent_id ?? ''}
          onChange={(value) => setItemForm((current) => ({ ...current, parent_id: value || null }))}
        >
          <MenuItem value="">{t('common.none')}</MenuItem>
          {allItems.items
            .filter((item) => item.id !== editingItem?.id)
            .map((item) => (
              <MenuItem key={item.id} value={item.id}>
                {translatedMenuItem(item, t)}
              </MenuItem>
            ))}
        </TextFieldRow>
        <TextFieldRow
          required
          label={t('common.title')}
          value={itemForm.title}
          onChange={(value) => setItemForm((current) => ({ ...current, title: value }))}
        />
        <TextFieldRow
          required
          label={t('common.code')}
          value={itemForm.code}
          onChange={(value) => setItemForm((current) => ({ ...current, code: value }))}
        />
        <TextFieldRow
          required
          label={t('common.path')}
          value={itemForm.path}
          onChange={(value) => setItemForm((current) => ({ ...current, path: value }))}
        />
        <TextFieldRow
          select
          label={t('common.icon')}
          value={itemForm.icon ?? ''}
          onChange={(value) => setItemForm((current) => ({ ...current, icon: value || null }))}
        >
          <MenuItem value="">{t('common.none')}</MenuItem>
          {NAV_ICON_OPTIONS.map((option) => (
            <MenuItem key={option} value={option}>
              {option}
            </MenuItem>
          ))}
        </TextFieldRow>
        <TextFieldRow
          label={t('common.caption')}
          value={itemForm.caption ?? ''}
          onChange={(value) => setItemForm((current) => ({ ...current, caption: value || null }))}
        />
        <TextFieldRow
          type="number"
          label={t('common.sortOrder')}
          value={itemForm.sort_order}
          onChange={(value) => setItemForm((current) => ({ ...current, sort_order: Number(value) }))}
        />
        <Box>
          <Label color="info" variant="soft" sx={{ mr: 1 }}>
            {t('helper.deepMatch')}
          </Label>
        </Box>
        <SwitchRow
          label={t('fields.deepMatch')}
          checked={itemForm.deep_match}
          onChange={(deepMatch) => setItemForm((current) => ({ ...current, deep_match: deepMatch }))}
        />
        <SwitchRow
          label={t('common.enabled')}
          checked={itemForm.enabled}
          onChange={(enabled) => setItemForm((current) => ({ ...current, enabled }))}
        />
      </ManagementDialog>

      <ConfirmDialog
        open={!!deleteSectionTarget}
        onClose={() => setDeleteSectionTarget(null)}
        title={t('dialogs.deleteMenuSection')}
        content={t('dialogs.deleteContent', { name: deleteSectionTarget?.subheader ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={confirmDeleteSection}>
            {t('common.delete')}
          </Button>
        }
      />

      <ConfirmDialog
        open={!!deleteItemTarget}
        onClose={() => setDeleteItemTarget(null)}
        title={t('dialogs.deleteMenuItem')}
        content={t('dialogs.deleteContent', { name: deleteItemTarget?.title ?? '' })}
        cancelText={t('common.cancel')}
        action={
          <Button variant="contained" color="error" onClick={confirmDeleteItem}>
            {t('common.delete')}
          </Button>
        }
      />
    </DashboardContent>
  );
}

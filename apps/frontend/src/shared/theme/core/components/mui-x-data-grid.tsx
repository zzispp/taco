import type { Theme, CSSObject, Components } from '@mui/material/styles';

import { varAlpha } from 'minimal-shared/utils';

import { gridClasses } from '@mui/x-data-grid';
import { listClasses } from '@mui/material/List';
import { paperClasses } from '@mui/material/Paper';
import { svgIconClasses } from '@mui/material/SvgIcon';
import { iconButtonClasses } from '@mui/material/IconButton';
import { listItemIconClasses } from '@mui/material/ListItemIcon';
import { linearProgressClasses } from '@mui/material/LinearProgress';
import { circularProgressClasses } from '@mui/material/CircularProgress';

import {
  MoreIcon,
  CloseIcon,
  FilterIcon,
  ExportIcon,
  SearchIcon,
  ArrowUpIcon,
  EyeCloseIcon,
  ArrowDownIcon,
  RemoveAllIcon,
  SeparatorIcon,
  ViewColumnsIcon,
} from './mui-x-data-grid-icons';

export {
  EyeIcon,
  ExportIcon,
  FilterIcon,
  EyeCloseIcon,
  ViewColumnsIcon,
  DensityCompactIcon,
  DensityStandardIcon,
  DensityComfortableIcon,
} from './mui-x-data-grid-icons';

// ----------------------------------------------------------------------

/* **********************************************************************
 * 🧩 Components
 * **********************************************************************/
const MuiDataGrid: Components<Theme>['MuiDataGrid'] = {
  // ▼▼▼▼▼▼▼▼ ⚙️ PROPS ▼▼▼▼▼▼▼▼
  defaultProps: {
    showToolbar: true,
    slots: {
      /* Column */
      columnSortedAscendingIcon: ArrowUpIcon,
      columnSortedDescendingIcon: ArrowDownIcon,
      columnMenuSortAscendingIcon: ArrowUpIcon,
      columnMenuIcon: MoreIcon,
      columnMenuFilterIcon: FilterIcon,
      columnMenuHideIcon: EyeCloseIcon,
      columnMenuSortDescendingIcon: ArrowDownIcon,
      columnMenuManageColumnsIcon: ViewColumnsIcon,
      columnSelectorIcon: ViewColumnsIcon,
      columnResizeIcon: SeparatorIcon,
      /* Filter */
      filterPanelDeleteIcon: CloseIcon,
      openFilterButtonIcon: FilterIcon,
      columnFilteredIcon: FilterIcon,
      filterPanelRemoveAllIcon: RemoveAllIcon,
      /* Export */
      exportIcon: ExportIcon,
      /* Quick filter */
      quickFilterIcon: SearchIcon,
      quickFilterClearIcon: CloseIcon,
    },
    slotProps: {
      baseSelect: {
        native: true,
      },
      loadingOverlay: {
        variant: 'skeleton',
      },
      columnsManagement: {
        searchInputProps: {
          size: 'medium',
        },
      },
    },
  },
  // ▼▼▼▼▼▼▼▼ 🎨 STYLE ▼▼▼▼▼▼▼▼
  styleOverrides: {
    root: ({ theme }) => {
      const baseStyles: CSSObject = {
        borderWidth: 0,
        backgroundColor: 'transparent',
      };

      return {
        '--unstable_DataGrid-radius': 0,
        '--unstable_DataGrid-headWeight': theme.typography.fontWeightSemiBold,
        ...theme.mixins.scrollbarStyles(theme),
        ...baseStyles,
      };
    },
    footerContainer: {
      minHeight: 'auto',
      borderTopStyle: 'dashed',
      [`& .${gridClasses.selectedRowCount}`]: {
        whiteSpace: 'nowrap',
      },
    },
    /**
     * @overlay
     */
    overlay: ({ theme }) => ({
      [`& .${linearProgressClasses.root}`]: {
        height: 3,
        borderRadius: 0,
        backgroundColor: varAlpha(theme.vars.palette.text.primaryChannel, 0.16),
        [`& .${linearProgressClasses.bar1}, .${linearProgressClasses.bar2}`]: {
          backgroundColor: theme.vars.palette.text.primary,
        },
      },
      [`& .${circularProgressClasses.root}`]: {
        color: theme.vars.palette.text.primary,
      },
    }),
    /**
     * @column
     */
    columnHeader: ({ theme }) => ({
      color: theme.vars.palette.text.secondary,
      backgroundColor: theme.vars.palette.background.neutral,
      [`&.${gridClasses['columnHeader--sorted']}, &.${gridClasses['columnHeader--sorted']} .${gridClasses.sortIcon}`]:
        {
          color: theme.vars.palette.text.primary,
        },
    }),
    /**
     * @cell
     */
    cell: ({ theme }) => ({
      borderTopStyle: 'dashed',
      '&:hover': {
        color: theme.vars.palette.primary.main,
      },
      [`&.${gridClasses['cell--editing']}`]: {
        boxShadow: 'none',
        backgroundColor: varAlpha(theme.vars.palette.primary.mainChannel, 0.08),
      },
      [`&.${gridClasses['cell--withLeftBorder']}`]: {
        borderLeftStyle: 'dashed',
      },
      [`&.${gridClasses['cell--withRightBorder']}`]: {
        borderRightStyle: 'dashed',
      },
    }),
    /**
     * @toolbar
     */
    toolbar: ({ theme }) => ({
      minHeight: 'auto',
      borderBottom: 'none',
      padding: theme.spacing(2),
    }),
    toolbarDivider: {
      display: 'none',
    },
    /**
     * @panel
     */
    panelContent: ({ theme }) => ({
      gap: theme.spacing(4),
      padding: theme.spacing(3, 2.5, 3, 2),
      [`&.${gridClasses.paper}`]: {
        ...theme.mixins.paperStyles(theme, { dropdown: true }),
        margin: 0,
        padding: 0,
      },
    }),
    panelFooter: ({ theme }) => ({
      padding: theme.spacing(1.5),
    }),
    menu: ({ theme }) => ({
      [`& .${paperClasses.root}`]: {
        ...theme.mixins.paperStyles(theme, { dropdown: true }),
      },
      [`& .${listClasses.root}`]: {
        padding: 0,
        [`& .${listItemIconClasses.root}`]: {
          minWidth: 0,
          marginRight: theme.spacing(2),
        },
      },
    }),
    /**
     * @panel column
     */
    columnsManagementHeader: ({ theme }) => ({
      paddingTop: theme.spacing(2.5),
    }),
    columnsManagement: ({ theme }) => ({
      gap: theme.spacing(0.5),
    }),
    columnsManagementFooter: ({ theme }) => ({
      borderTopStyle: 'dashed',
      paddingTop: theme.spacing(1.5),
      paddingBottom: theme.spacing(1.5),
    }),
    /**
     * @panel filter
     */
    filterFormDeleteIcon: ({ theme }) => ({
      [`& .${iconButtonClasses.root}`]: {
        padding: '5px',
        backgroundColor: varAlpha(theme.vars.palette.grey['500Channel'], 0.16),
        [`& .${svgIconClasses.root}`]: { width: 16, height: 16 },
      },
    }),
  },
};

/* **********************************************************************
 * 🚀 Export
 * **********************************************************************/
export const dataGrid: Components<Theme> = {
  MuiDataGrid,
};

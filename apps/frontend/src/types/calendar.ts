import type { IDatePickerControl } from './common';

// ----------------------------------------------------------------------

export type ICalendarFilters = {
  colors: string[];
  startDate: IDatePickerControl;
  endDate: IDatePickerControl;
};

export type ICalendarDate = string | number;

export type ICalendarRange = {
  start: ICalendarDate;
  end: ICalendarDate;
} | null;

export type ICalendarEvent = {
  id: string;
  color: string;
  title: string;
  allDay: boolean;
  description: string;
  end: ICalendarDate;
  start: ICalendarDate;
};

export type ListView = 'list' | 'listDay' | 'listWeek' | 'listMonth' | 'listYear';
export type DayGridView = 'dayGrid' | 'dayGridDay' | 'dayGridWeek' | 'dayGridMonth' | 'dayGridYear';
export type TimeGridView = 'timeGrid' | 'timeGridDay' | 'timeGridWeek';
export type ICalendarView = ListView | DayGridView | TimeGridView;

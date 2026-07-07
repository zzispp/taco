'use client';

import type { Dayjs } from 'dayjs';

import { useState, useCallback } from 'react';

import { fIsAfter, fDateRangeShortLabel } from 'src/shared/lib/format-time';

// ----------------------------------------------------------------------

export type DateValue = Dayjs | null;

export type UseDateRangePickerReturn = {
  startDate: DateValue;
  endDate: DateValue;
  onChangeStartDate: (newValue: DateValue) => void;
  onChangeEndDate: (newValue: DateValue) => void;
  /********/
  open: boolean;
  onOpen?: () => void;
  onClose: () => void;
  onReset?: () => void;
  /********/
  selected?: boolean;
  error?: boolean;
  /********/
  label?: string;
  shortLabel?: string;
  /********/
  title?: string;
  variant?: 'calendar' | 'input';
  /********/
  setStartDate?: React.Dispatch<React.SetStateAction<DateValue>>;
  setEndDate?: React.Dispatch<React.SetStateAction<DateValue>>;
};

export function useDateRangePicker(start: DateValue, end: DateValue): UseDateRangePickerReturn {
  const [open, setOpen] = useState(false);
  const [endDate, setEndDate] = useState<DateValue>(end);
  const [startDate, setStartDate] = useState<DateValue>(start);
  const error = fIsAfter(startDate, endDate);
  const onOpen = useCallback(() => {
    setOpen(true);
  }, []);
  const onClose = useCallback(() => {
    setOpen(false);
  }, []);
  const onChangeStartDate = useCallback((newValue: DateValue) => {
    setStartDate(newValue);
  }, []);

  const onChangeEndDate = useCallback(
    (newValue: DateValue) => {
      if (error) {
        setEndDate(null);
      }
      setEndDate(newValue);
    },
    [error]
  );

  const onReset = useCallback(() => {
    setStartDate(null);
    setEndDate(null);
  }, []);

  return dateRangePickerState({
    startDate,
    endDate,
    onChangeStartDate,
    onChangeEndDate,
    /********/
    onOpen,
    onClose,
    onReset,
    setStartDate,
    setEndDate,
    open,
    error,
  });
}

type DateRangePickerStateOptions = Pick<
  UseDateRangePickerReturn,
  | 'startDate'
  | 'endDate'
  | 'onChangeStartDate'
  | 'onChangeEndDate'
  | 'open'
  | 'onOpen'
  | 'onClose'
  | 'onReset'
  | 'error'
  | 'setStartDate'
  | 'setEndDate'
>;

function dateRangePickerState(options: DateRangePickerStateOptions): UseDateRangePickerReturn {
  const { startDate, endDate } = options;

  return {
    ...options,
    selected: !!startDate && !!endDate,
    label: fDateRangeShortLabel(startDate, endDate, true),
    shortLabel: fDateRangeShortLabel(startDate, endDate),
  };
}

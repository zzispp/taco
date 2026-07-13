import { it, expect, describe } from 'vitest';

import {
  changeCronEditorMode,
  cronEditorExpression,
  createCronEditorModel,
} from './cron-builder-parser';

const CURRENT_YEAR = 2026;

describe('Cron editor model', () => {
  it('uses builder mode only for losslessly representable expressions', () => {
    const model = createCronEditorModel('0 5 2 ? * 2 2026', CURRENT_YEAR);

    expect(model.mode).toBe('builder');
    expect(cronEditorExpression(model)).toBe('0 5 2 ? * 2 2026');
  });

  it.each(['1-10/2 * * * * ?', '1,3-5 * * * * ?', '0 0 12 ? JAN MON'])(
    'keeps advanced expression %s in custom mode without rewriting it',
    (expression) => {
      const model = createCronEditorModel(expression, CURRENT_YEAR);

      expect(model.mode).toBe('custom');
      expect(cronEditorExpression(model)).toBe(expression);
      expect(cronEditorExpression(model)).not.toContain('NaN');
    }
  );

  it('preserves whitespace until the expression is edited', () => {
    const expression = '  0   5 2 ? * 2 2026  ';
    const model = createCronEditorModel(expression, CURRENT_YEAR);

    expect(model.mode).toBe('builder');
    expect(cronEditorExpression(model)).toBe(expression);
  });

  it('does not replace an advanced custom expression when builder mode is requested', () => {
    const expression = '1-10/2 * * * * ?';
    const model = createCronEditorModel(expression, CURRENT_YEAR);
    const switched = changeCronEditorMode(model, 'builder', CURRENT_YEAR);

    expect(switched.mode).toBe('custom');
    expect(cronEditorExpression(switched)).toBe(expression);
  });
});

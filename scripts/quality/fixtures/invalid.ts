import type { Role } from 'src/entities/role';

export function invalid(first: boolean, second: boolean, third: boolean, fourth: Role): boolean {
  if (first) {
    if (second) {
      if (third) {
        if (fourth) {
          return true;
        }
      }
    }
  }
  return false;
}

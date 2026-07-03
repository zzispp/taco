import { usePathname as useNextPathname } from 'next/navigation';

export function usePathname() {
  return useNextPathname() ?? '';
}

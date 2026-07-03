import { useSearchParams as useNextSearchParams } from 'next/navigation';

export function useSearchParams() {
  const searchParams = useNextSearchParams();
  return new URLSearchParams(searchParams?.toString() ?? '');
}

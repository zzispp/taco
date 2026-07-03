import { MainLayout } from 'src/widgets/main-shell';

// ----------------------------------------------------------------------

type Props = {
  children: React.ReactNode;
};

export default function Layout({ children }: Props) {
  return <MainLayout>{children}</MainLayout>;
}

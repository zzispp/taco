export type DocumentLocation = Pick<Location, 'replace'>;

export function replaceDocumentLocation(location: DocumentLocation, href: string) {
  location.replace(href);
}

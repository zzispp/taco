import type { UploadProgress } from './upload-progress';

const DIGEST_PREVIEW_HEAD_LENGTH = 10;
const DIGEST_PREVIEW_TAIL_LENGTH = 8;

export type UploadQueueStatus =
  | 'queued'
  | 'hashing'
  | 'uploading'
  | 'completing'
  | 'completed'
  | 'failed';

export type UploadQueueItem = Readonly<{
  id: string;
  file: File;
  relativePath: string;
  digest: string | null;
  progress: UploadProgress | null;
  status: UploadQueueStatus;
}>;

export type UploadQueueTreeRow = Readonly<{
  id: string;
  kind: 'folder' | 'file';
  name: string;
  depth: number;
  size: number;
  item?: UploadQueueItem;
}>;

type UploadFileWithPath = File & Readonly<{ path?: string }>;

type UploadTreeNode = {
  name: string;
  path: string;
  item?: UploadQueueItem;
  children: Map<string, UploadTreeNode>;
};

export function createUploadQueue(files: readonly File[]): readonly UploadQueueItem[] {
  return files.map((file, index) => {
    const relativePath = uploadRelativePath(file);
    return {
      id: uploadQueueItemId(relativePath, file, index),
      file,
      relativePath,
      digest: null,
      progress: null,
      status: 'queued',
    };
  });
}

export function uploadRelativePath(file: File): string {
  const fileWithPath = file as UploadFileWithPath;
  const rawPath = file.webkitRelativePath || fileWithPath.path || file.name;
  return normalizeUploadPath(rawPath);
}

export function uploadParentSegments(
  item: Pick<UploadQueueItem, 'relativePath'>
): readonly string[] {
  return item.relativePath.split('/').slice(0, -1);
}

export function canSubmitUploadQueue(items: readonly UploadQueueItem[]): boolean {
  return items.some((item) => item.status === 'queued' || item.status === 'failed');
}

export function isUploadQueueBusy(items: readonly UploadQueueItem[]): boolean {
  return items.some(
    (item) =>
      item.status === 'hashing' || item.status === 'uploading' || item.status === 'completing'
  );
}

export function buildUploadQueueTree(
  items: readonly UploadQueueItem[]
): readonly UploadQueueTreeRow[] {
  const root = createUploadTreeNode('', '');
  items.forEach((item) => addUploadTreeItem(root, item));
  return flattenUploadTree(root, 0);
}

export function truncateUploadDigest(digest: string): string {
  const previewLength = DIGEST_PREVIEW_HEAD_LENGTH + DIGEST_PREVIEW_TAIL_LENGTH;
  if (digest.length <= previewLength) return digest;
  return `${digest.slice(0, DIGEST_PREVIEW_HEAD_LENGTH)}...${digest.slice(-DIGEST_PREVIEW_TAIL_LENGTH)}`;
}

function normalizeUploadPath(rawPath: string): string {
  const path = rawPath.replace(/^\/+|\/+$/g, '');
  const segments = (path.startsWith('./') ? path.slice(2) : path).split('/');
  if (!path || segments.some(isInvalidUploadPathSegment)) {
    throw new Error('Invalid upload path');
  }
  return segments.join('/');
}

function isInvalidUploadPathSegment(segment: string): boolean {
  return !segment || segment === '.' || segment === '..' || segment.includes('\\');
}

function uploadQueueItemId(relativePath: string, file: File, index: number): string {
  return `${crypto.randomUUID()}:${relativePath}:${file.size}:${file.lastModified}:${index}`;
}

function createUploadTreeNode(name: string, path: string): UploadTreeNode {
  return { name, path, children: new Map() };
}

function addUploadTreeItem(root: UploadTreeNode, item: UploadQueueItem) {
  const segments = item.relativePath.split('/');
  const fileName = segments.pop();
  if (!fileName) throw new Error('Invalid upload path');
  const parent = segments.reduce((node, segment) => addUploadTreeFolder(node, segment), root);
  parent.children.set(`file:${item.id}`, {
    ...createUploadTreeNode(fileName, item.relativePath),
    item,
  });
}

function addUploadTreeFolder(node: UploadTreeNode, name: string): UploadTreeNode {
  const folderKey = `folder:${name}`;
  const existing = node.children.get(folderKey);
  if (existing) return existing;
  const path = node.path ? `${node.path}/${name}` : name;
  const folder = createUploadTreeNode(name, path);
  node.children.set(folderKey, folder);
  return folder;
}

function flattenUploadTree(node: UploadTreeNode, depth: number): UploadQueueTreeRow[] {
  return sortedUploadTreeNodes(node.children).flatMap((child) =>
    flattenUploadTreeNode(child, depth)
  );
}

function flattenUploadTreeNode(node: UploadTreeNode, depth: number): UploadQueueTreeRow[] {
  if (node.item) {
    return [
      {
        id: node.item.id,
        kind: 'file',
        name: node.name,
        depth,
        size: node.item.file.size,
        item: node.item,
      },
    ];
  }
  return [
    {
      id: `folder:${node.path}`,
      kind: 'folder',
      name: node.name,
      depth,
      size: uploadTreeSize(node),
    },
    ...flattenUploadTree(node, depth + 1),
  ];
}

function sortedUploadTreeNodes(children: Map<string, UploadTreeNode>): UploadTreeNode[] {
  return [...children.values()].sort((left, right) => {
    if (Boolean(left.item) !== Boolean(right.item)) return left.item ? 1 : -1;
    return left.name.localeCompare(right.name);
  });
}

function uploadTreeSize(node: UploadTreeNode): number {
  if (node.item) return node.item.file.size;
  return [...node.children.values()].reduce((total, child) => total + uploadTreeSize(child), 0);
}

export type ClipboardWriter = Readonly<{
  writeText: (value: string) => Promise<void>;
}>;

export type ClipboardFeedback = Readonly<{
  success: () => void;
  failure: () => void;
}>;

export async function copyTextToClipboard(value: string, writer?: ClipboardWriter) {
  await (writer ?? browserClipboard()).writeText(value);
}

export async function copyTextWithFeedback(
  value: string,
  feedback: ClipboardFeedback,
  writer?: ClipboardWriter
) {
  try {
    await copyTextToClipboard(value, writer);
    feedback.success();
    return true;
  } catch {
    feedback.failure();
    return false;
  }
}

function browserClipboard(): ClipboardWriter {
  if (typeof navigator === 'undefined' || !navigator.clipboard) {
    throw new Error('Clipboard API is unavailable');
  }
  return navigator.clipboard;
}

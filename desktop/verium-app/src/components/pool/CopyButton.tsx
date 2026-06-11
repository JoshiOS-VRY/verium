import { Check, Copy } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { useCopyToClipboard } from '@/hooks/useCopyToClipboard';

export function CopyButton({ value, label = 'Copy' }: { value: string; label?: string }) {
  const { copied, copy } = useCopyToClipboard();
  return (
    <Button
      type="button"
      variant="ghost"
      className="h-8 gap-1.5 px-2 text-xs"
      onClick={() => void copy(value)}
      aria-label={label}
    >
      {copied ? (
        <Check className="h-3.5 w-3.5 text-success" aria-hidden />
      ) : (
        <Copy className="h-3.5 w-3.5" aria-hidden />
      )}
      {copied ? 'Copied' : 'Copy'}
    </Button>
  );
}

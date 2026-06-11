import { useState } from 'react';
import { Lock } from 'lucide-react';
import { Button } from '@/components/ui/Button';

interface SendPassphrasePromptProps {
  open: boolean;
  title?: string;
  onVerified: (passphrase: string) => void;
  onCancel: () => void;
}

export function SendPassphrasePrompt({
  open,
  title = 'Confirm send with passphrase',
  onVerified,
  onCancel,
}: SendPassphrasePromptProps) {
  const [passphrase, setPassphrase] = useState('');
  const [error, setError] = useState<string | null>(null);

  if (!open) return null;

  const submit = () => {
    if (!passphrase.trim()) {
      setError('Enter your wallet passphrase.');
      return;
    }
    const verified = passphrase;
    setPassphrase('');
    setError(null);
    onVerified(verified);
  };

  return (
    <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/60 p-4 backdrop-blur-sm">
      <div className="w-full max-w-sm rounded-xl border border-border bg-bg-panel p-5 shadow-2xl">
        <div className="mb-4 flex items-center gap-3">
          <Lock className="h-5 w-5 text-accent" />
          <h2 className="text-base font-semibold">{title}</h2>
        </div>
        <p className="mb-3 text-sm text-fg-muted">
          Enter your wallet passphrase to authorize this payment. It is required for every send when
          two-factor authentication is disabled.
        </p>
        <input
          type="password"
          autoComplete="current-password"
          autoFocus
          value={passphrase}
          onChange={(e) => {
            setPassphrase(e.target.value);
            setError(null);
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') submit();
          }}
          placeholder="Wallet passphrase"
          className="mb-3 h-10 w-full rounded-md border border-border bg-bg-subtle px-3 text-sm outline-none focus:border-accent"
        />
        {error && <p className="mb-3 text-xs text-danger">{error}</p>}
        <div className="flex justify-end gap-2">
          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              setPassphrase('');
              setError(null);
              onCancel();
            }}
          >
            Cancel
          </Button>
          <Button size="sm" disabled={!passphrase.trim()} onClick={submit}>
            Confirm send
          </Button>
        </div>
      </div>
    </div>
  );
}

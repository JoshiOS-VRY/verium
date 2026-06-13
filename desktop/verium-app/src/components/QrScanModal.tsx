import { useCallback, useEffect, useRef, useState } from 'react';
import QrScanner from 'qr-scanner';
import qrScannerWorkerUrl from 'qr-scanner/qr-scanner-worker.min.js?url';
import { X } from 'lucide-react';
import { Button } from '@/components/ui/Button';
import { ScanSuccessToast } from '@/components/ScanSuccessToast';
import { parsePaymentUri } from '@/lib/security/client';

QrScanner.WORKER_PATH = qrScannerWorkerUrl;

interface QrScanModalProps {
  open: boolean;
  onClose: () => void;
  onScan: (address: string, amount?: number) => void;
}

function cameraErrorMessage(error: unknown): string {
  if (error instanceof DOMException) {
    if (error.name === 'NotAllowedError' || error.name === 'PermissionDeniedError') {
      return 'Camera permission denied. Open Settings → Vericonomy Wallet and enable Camera, then try again.';
    }
    if (error.name === 'NotFoundError' || error.name === 'DevicesNotFoundError') {
      return 'No camera found on this device.';
    }
    if (error.name === 'NotReadableError') {
      return 'Camera is in use by another app. Close it and try again.';
    }
  }
  return 'Camera unavailable. Paste the address manually instead.';
}

export function QrScanModal({ open, onClose, onScan }: QrScanModalProps) {
  const videoRef = useRef<HTMLVideoElement>(null);
  const scannerRef = useRef<QrScanner | null>(null);
  const handledRef = useRef(false);
  const onScanRef = useRef(onScan);
  const [error, setError] = useState<string | null>(null);
  const [starting, setStarting] = useState(false);
  const [showSuccess, setShowSuccess] = useState(false);

  onScanRef.current = onScan;

  const finishSuccess = useCallback(() => {
    setShowSuccess(false);
    onClose();
  }, [onClose]);

  const handleScanResult = useCallback((address: string, amount?: number) => {
    if (handledRef.current) return;
    handledRef.current = true;

    if (scannerRef.current) {
      void scannerRef.current.stop();
    }

    onScanRef.current(address, amount);
    setShowSuccess(true);
  }, []);

  useEffect(() => {
    if (!open) {
      setError(null);
      setStarting(false);
      setShowSuccess(false);
      handledRef.current = false;
      return;
    }

    let cancelled = false;
    let scanner: QrScanner | null = null;

    const startScanner = async () => {
      const video = videoRef.current;
      if (!video) return;

      setError(null);
      setStarting(true);

      if (!navigator.mediaDevices?.getUserMedia) {
        setError('Camera is not available in this app build.');
        setStarting(false);
        return;
      }

      try {
        const hasCamera = await QrScanner.hasCamera();
        if (!hasCamera) {
          setError('No camera found on this device.');
          setStarting(false);
          return;
        }
      } catch {
        // hasCamera can fail on some WebViews; still try getUserMedia.
      }

      if (cancelled) return;

      scanner = new QrScanner(
        video,
        (result) => {
          void (async () => {
            try {
              const parsed = await parsePaymentUri(result.data);
              handleScanResult(parsed.address, parsed.amount ?? undefined);
            } catch {
              if (result.data.length > 20) {
                handleScanResult(result.data.trim());
              }
            }
          })();
        },
        { highlightScanRegion: true, preferredCamera: 'environment' }
      );
      scannerRef.current = scanner;

      try {
        await scanner.start();
        if (!cancelled) setStarting(false);
      } catch (err) {
        if (!cancelled) {
          setError(cameraErrorMessage(err));
          setStarting(false);
        }
      }
    };

    requestAnimationFrame(() => {
      void startScanner();
    });

    return () => {
      cancelled = true;
      if (scanner) {
        void scanner.stop();
        scanner.destroy();
      }
      scannerRef.current = null;
    };
  }, [open]);

  if (!open && !showSuccess) return null;

  return (
    <>
      {open && !showSuccess && (
        <div
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 p-4 backdrop-blur-sm"
          onMouseDown={(e) => {
            if (e.target === e.currentTarget) onClose();
          }}
        >
          <div
            role="dialog"
            aria-modal="true"
            aria-labelledby="qr-scan-title"
            className="w-full max-w-md rounded-xl border border-border bg-bg-panel p-4 shadow-2xl"
          >
            <div className="mb-3 flex items-center justify-between">
              <h2 id="qr-scan-title" className="text-base font-semibold">
                Scan QR code
              </h2>
              <button
                type="button"
                onClick={onClose}
                aria-label="Close"
                className="flex h-9 w-9 items-center justify-center rounded-lg text-fg-muted active:bg-bg-subtle"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
            <div className="relative overflow-hidden rounded-lg bg-black">
              <video
                ref={videoRef}
                playsInline
                muted
                className="aspect-square w-full bg-black object-cover"
              />
              {starting && !error && (
                <div className="absolute inset-0 flex items-center justify-center bg-black/50 text-sm text-white">
                  Starting camera…
                </div>
              )}
            </div>
            {error && <p className="mt-2 text-sm text-danger">{error}</p>}
            <p className="mt-2 text-xs text-fg-muted">
              Point your camera at a payment QR code. Works best in good lighting.
            </p>
            <div className="mt-3 flex justify-end">
              <Button size="sm" variant="secondary" onClick={onClose}>
                Cancel
              </Button>
            </div>
          </div>
        </div>
      )}

      {showSuccess && (
        <ScanSuccessToast
          message="Address copied successfully"
          durationMs={1000}
          onDone={finishSuccess}
        />
      )}
    </>
  );
}

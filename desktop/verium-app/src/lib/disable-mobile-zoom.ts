/** Block pinch-to-zoom and double-tap zoom in mobile WebViews (WKWebView ignores viewport alone). */
export function disableMobileZoom(): void {
  const blockGesture = (event: Event) => {
    event.preventDefault();
  };

  document.addEventListener('gesturestart', blockGesture, { passive: false });
  document.addEventListener('gesturechange', blockGesture, { passive: false });
  document.addEventListener('gestureend', blockGesture, { passive: false });

  document.addEventListener(
    'touchmove',
    (event) => {
      if (event.touches.length > 1) {
        event.preventDefault();
      }
    },
    { passive: false }
  );
}

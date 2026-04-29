document.addEventListener("DOMContentLoaded", () => {
  const container = document.scrollingElement;

  if (container) {
    window.addEventListener(
      "wheel",
      (e) => {
        if (e.deltaY !== 0) {
          e.preventDefault();

          container.scrollBy({
            left: -e.deltaY,
            behavior: "smooth",
          });
        }
      },
      { passive: false },
    );
  }
});

const INITIAL_HASH_POSITIONING_ATTRIBUTE = "data-initial-hash-positioning";

const applyInitialHashBootstrap = (attribute: string) => {
  if (
    window.location.pathname === "/" &&
    window.location.hash === "#flight-search"
  ) {
    document.documentElement.setAttribute(attribute, "");
  }
};

const INITIAL_HASH_BOOTSTRAP_SCRIPT = `(${applyInitialHashBootstrap.toString()})(${JSON.stringify(INITIAL_HASH_POSITIONING_ATTRIBUTE)});`;

const runInitialHashBootstrap = () =>
  applyInitialHashBootstrap(INITIAL_HASH_POSITIONING_ATTRIBUTE);

const clearInitialHashPositioning = () =>
  document.documentElement.removeAttribute(INITIAL_HASH_POSITIONING_ATTRIBUTE);

export {
  clearInitialHashPositioning,
  INITIAL_HASH_BOOTSTRAP_SCRIPT,
  INITIAL_HASH_POSITIONING_ATTRIBUTE,
  runInitialHashBootstrap,
};

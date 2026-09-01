const INITIAL_HASH_POSITIONING_ATTRIBUTE = "data-initial-hash-positioning";
const SUPPORTED_HOME_HASHES = new Set([
  "#cabins",
  "#experience",
  "#explore",
  "#flight-search",
  "#offers",
  "#top",
]);

const applyInitialHashBootstrap = (attribute: string) => {
  window.history.scrollRestoration = "manual";

  if (
    window.location.pathname === "/" &&
    SUPPORTED_HOME_HASHES.has(window.location.hash)
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

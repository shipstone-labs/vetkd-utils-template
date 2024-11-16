declare module "*.wasm" {
  const value: () => Promise<WebAssembly.Module>;
  export default value;
}

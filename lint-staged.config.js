module.exports = {
  "*.{tsx,ts,json}": () => "yarn check:types",
  "*.{tsx,ts,js,cjs}": ["eslint --cache --fix", "prettier --write"],
  "*.{mdx,json}": ["prettier --write"],
};

const { defineConfig } = require("pollen-css/utils");

// TODO: generate pollen with a Rollup plugin

module.exports = (pollen) => defineConfig({
    output: "./src/styles/pollen.css",
    modules: {
        font: {
            ...pollen.font,
            sans: `'Inter', ${pollen.font.sans}`,
            serif: `'Playfair Display', ${pollen.font.serif}`,
        }
    }
});